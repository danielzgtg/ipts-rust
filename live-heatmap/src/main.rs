// Parts were derived from the vulkano triangle example
// The original is
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.
// The above licenses shall only apply to the parts derived from that source

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use vulkano::buffer::{BufferAccess, BufferUsage, DeviceLocalBuffer, ImmutableBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState, SubpassContents};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor::{DescriptorSet, PipelineLayoutAbstract};
use vulkano::device::Device;
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::view::ImageView;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::impl_vertex;
use vulkano::pipeline::vertex::{
    BufferlessDefinition, BufferlessVertices, OneVertexOneInstanceDefinition,
};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::single_pass_renderpass;
use vulkano::swapchain::{
    AcquireError, ColorSpace, CompositeAlpha, FullscreenExclusive, PresentMode, Surface,
    SurfaceTransform, Swapchain, SwapchainAcquireFuture, SwapchainCreationError,
};
use vulkano::sync::{FlushError, GpuFuture, SharingMode};
use vulkano_win::VkSurfaceBuild;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use engine::Engine;
use ipts_dev::{HeaderAndBuffer, Ipts, IptsExt};
use utils::{get_heatmap, Pointers, Report};

macro_rules! vs {
    ($id: ident, $path: expr) => {
        mod $id {
            vulkano_shaders::shader! {
                ty: "vertex",
                path: $path,
            }
            #[allow(dead_code)]
            const _ENSURE_VULKANO_RECOMPILES_CHANGES: &str = include_str!(concat!("../", $path));
        }
    };
}

macro_rules! fs {
    ($id: ident, $path: expr) => {
        mod $id {
            vulkano_shaders::shader! {
                ty: "fragment",
                path: $path,
            }
            #[allow(dead_code)]
            const _ENSURE_VULKANO_RECOMPILES_CHANGES: &str = include_str!(concat!("../", $path));
        }
    };
}

vs!(vs_cursor, "src/shaders/vs_cursor.glsl");
vs!(vs_fullscreen, "src/shaders/vs_fullscreen.glsl");
fs!(fs_cursor, "src/shaders/fs_cursor.glsl");
fs!(fs_heatmap, "src/shaders/fs_heatmap.glsl");

#[derive(Copy, Clone, Default, Debug)]
struct Vertex {
    vertex_position: [f32; 2],
}
impl_vertex!(Vertex, vertex_position);

#[derive(Copy, Clone, Default, Debug)]
struct Instance {
    instance_data: [u32; 3],
}
impl_vertex!(Instance, instance_data);

pub struct Shaders {
    pub vs_cursor: vs_cursor::Shader,
    pub vs_fullscreen: vs_fullscreen::Shader,
    pub fs_cursor: fs_cursor::Shader,
    pub fs_heatmap: fs_heatmap::Shader,
}

impl Shaders {
    pub fn new(device: &Arc<Device>) -> Shaders {
        Shaders {
            vs_cursor: vs_cursor::Shader::load(device.clone()).unwrap(),
            vs_fullscreen: vs_fullscreen::Shader::load(device.clone()).unwrap(),
            fs_cursor: fs_cursor::Shader::load(device.clone()).unwrap(),
            fs_heatmap: fs_heatmap::Shader::load(device.clone()).unwrap(),
        }
    }
}

type MyRenderPass = Arc<dyn RenderPassAbstract + Send + Sync>;
type MyPipelineLayout = Box<dyn PipelineLayoutAbstract + Send + Sync>;

struct State {
    engine: Engine,
    running: Arc<AtomicBool>,
    new_data: Arc<Mutex<[u8; 2816]>>,
    reset_sensor: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
    pointers: Pointers,
    positions: [(u32, u32); 10],
    positions_length: usize,
    surface: Arc<Surface<Window>>,
    swapchain: Arc<Swapchain<Window>>,
    renderpass: MyRenderPass,
    pipeline: Arc<GraphicsPipeline<BufferlessDefinition, MyPipelineLayout, MyRenderPass>>,
    viewport_uniforms: Arc<DeviceLocalBuffer<fs_heatmap::ty::VP>>,
    new_viewport_uniforms: Option<fs_heatmap::ty::VP>,
    cursor_pipeline: Arc<
        GraphicsPipeline<
            OneVertexOneInstanceDefinition<Vertex, Instance>,
            MyPipelineLayout,
            MyRenderPass,
        >,
    >,
    vertex_positions: Arc<ImmutableBuffer<[Vertex; 4]>>,
    instance_positions: Arc<DeviceLocalBuffer<[Instance; 10]>>,
    new_instance_positions: Option<[Instance; 10]>,
    descriptor_set: Arc<dyn DescriptorSet + Send + Sync>,
    dynamic_state: DynamicState,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

fn background_thread(
    running: Arc<AtomicBool>,
    new_data: Arc<Mutex<[u8; 2816]>>,
    reset_sensor: Arc<AtomicBool>,
) -> JoinHandle<()> {
    let mut last_multitouch = Instant::now();
    std::thread::spawn(move || 'outer: loop {
        let mut ipts = Ipts::new();
        let mut buf = [0u8; 16384];

        loop {
            ipts.wait_for_doorbell(Instant::now() - last_multitouch < Duration::from_secs(1));
            ipts.read(&mut buf);

            let parsed = HeaderAndBuffer::from(&buf);
            if parsed.typ == 3 && parsed.size == 3500 && parsed.data[0] == 0x0B {
                let data = get_heatmap((&parsed.data[..3500]).try_into().unwrap());
                new_data.lock().unwrap().copy_from_slice(data);
                last_multitouch = Instant::now();
            }

            ipts.send_feedback();

            if reset_sensor.load(Ordering::Acquire) {
                reset_sensor.store(false, Ordering::Release);
                ipts.send_reset();
                std::thread::sleep(Duration::from_secs(1));
                break;
            }

            if !running.load(Ordering::Acquire) {
                break 'outer;
            };
        }
    })
}

fn create_framebuffers(
    images: &[Arc<SwapchainImage<Window>>],
    renderpass: &Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
    result: &mut Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
) {
    result.clear();
    dynamic_state.viewports.as_deref_mut().unwrap()[0].dimensions = {
        let dimensions = images[0].dimensions();
        [dimensions[0] as f32, dimensions[1] as f32]
    };
    for image in images.iter() {
        result.push(Arc::new(
            Framebuffer::start(renderpass.clone())
                .add(ImageView::new(image.clone()).unwrap())
                .unwrap()
                .build()
                .unwrap(),
        ));
    }
}

fn get_new_viewport_uniforms(new_size: PhysicalSize<u32>) -> fs_heatmap::ty::VP {
    fs_heatmap::ty::VP {
        viewport: [new_size.width as f32, new_size.height as f32],
    }
}

impl State {
    fn new(engine: Engine, event_loop: &EventLoop<()>) -> State {
        let running = Arc::new(AtomicBool::new(true));
        let new_data = Arc::new(Mutex::new([0u8; 2816]));
        let reset_sensor = Arc::new(AtomicBool::new(false));
        let join_handle = Some(background_thread(
            running.clone(),
            new_data.clone(),
            reset_sensor.clone(),
        ));
        let surface = WindowBuilder::new()
            .with_title("IPTS")
            .build_vk_surface(&event_loop, engine.vk())
            .unwrap();
        let shaders = Shaders::new(&engine.device());
        let (swapchain, images) = Swapchain::new(
            engine.device(),
            surface.clone(),
            3,
            Format::B8G8R8A8Srgb,
            surface.window().inner_size().into(),
            1,
            ImageUsage {
                transfer_destination: true,
                color_attachment: true,
                ..ImageUsage::none()
            },
            SharingMode::Exclusive,
            SurfaceTransform::Identity,
            CompositeAlpha::Opaque,
            PresentMode::Fifo,
            FullscreenExclusive::Default,
            true,
            ColorSpace::SrgbNonLinear,
        )
        .unwrap();
        let renderpass: Arc<dyn RenderPassAbstract + Send + Sync> = Arc::new(
            single_pass_renderpass!(
                engine.device(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: Format::B8G8R8A8Srgb,
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
                }
            )
            .unwrap(),
        );
        let pipeline = Arc::new(
            GraphicsPipeline::start()
                .viewports_dynamic_scissors_irrelevant(1)
                .vertex_shader(shaders.vs_fullscreen.main_entry_point(), ())
                .fragment_shader(shaders.fs_heatmap.main_entry_point(), ())
                .render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
                .build(engine.device())
                .unwrap(),
        );
        let viewport_uniforms = DeviceLocalBuffer::new(
            engine.device(),
            BufferUsage {
                transfer_destination: true,
                uniform_buffer: true,
                ..BufferUsage::none()
            },
            std::iter::once(engine.queue().family()),
        )
        .unwrap();
        let new_viewport_uniforms = Some(get_new_viewport_uniforms(surface.window().inner_size()));
        let cursor_pipeline = Arc::new(
            GraphicsPipeline::start()
                .viewports_dynamic_scissors_irrelevant(1)
                .triangle_strip()
                .vertex_input(OneVertexOneInstanceDefinition::<Vertex, Instance>::new())
                .vertex_shader(shaders.vs_cursor.main_entry_point(), ())
                .fragment_shader(shaders.fs_cursor.main_entry_point(), ())
                .blend_alpha_blending()
                .render_pass(Subpass::from(renderpass.clone(), 0).unwrap())
                .build(engine.device())
                .unwrap(),
        );
        let (vertex_positions, future) = ImmutableBuffer::from_data(
            [
                Vertex {
                    vertex_position: [-0.1, 0.1],
                },
                Vertex {
                    vertex_position: [-0.1, -0.1],
                },
                Vertex {
                    vertex_position: [0.1, 0.1],
                },
                Vertex {
                    vertex_position: [0.1, -0.1],
                },
            ],
            BufferUsage {
                transfer_destination: true,
                vertex_buffer: true,
                ..BufferUsage::none()
            },
            engine.queue(),
        )
        .unwrap();
        future
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();
        let instance_positions = DeviceLocalBuffer::new(
            engine.device(),
            BufferUsage {
                transfer_destination: true,
                vertex_buffer: true,
                ..BufferUsage::none()
            },
            std::iter::once(engine.queue().family()),
        )
        .unwrap();
        let new_instance_positions = None;
        let descriptor_set = Arc::new(
            PersistentDescriptorSet::start(
                pipeline.layout().descriptor_set_layout(0).unwrap().clone(),
            )
            .add_buffer(engine.get_parsed_buffer())
            .unwrap()
            .add_buffer(viewport_uniforms.clone())
            .unwrap()
            .build()
            .unwrap(),
        );
        let mut dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [0.0, 0.0],
                depth_range: 0.0..1.0,
            }]),
            ..DynamicState::default()
        };
        let mut framebuffers = Vec::with_capacity(3);
        create_framebuffers(&images, &renderpass, &mut dynamic_state, &mut framebuffers);
        State {
            engine,
            running,
            new_data,
            reset_sensor,
            join_handle,
            pointers: Pointers::new(),
            positions: [(0, 0); 10],
            positions_length: 0,
            surface,
            swapchain,
            renderpass,
            pipeline,
            viewport_uniforms,
            new_viewport_uniforms,
            cursor_pipeline,
            vertex_positions,
            instance_positions,
            new_instance_positions,
            descriptor_set,
            dynamic_state,
            framebuffers,
            previous_frame_end: None,
        }
    }

    fn cleanup(&mut self) {
        self.running.store(false, Ordering::Release);
        self.join_handle.take().unwrap().join().unwrap();
    }

    fn resize_loop(&mut self, initial_size: PhysicalSize<u32>) {
        if !self.resize(initial_size) {
            while !self.resize(self.surface.window().inner_size()) {}
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) -> bool {
        let (swapchain, images) = match self.swapchain.recreate_with_dimensions(new_size.into()) {
            Ok(x) => x,
            Err(SwapchainCreationError::UnsupportedDimensions) => return false,
            Err(x) => panic!("{:?}", x),
        };
        create_framebuffers(
            &images,
            &self.renderpass,
            &mut self.dynamic_state,
            &mut self.framebuffers,
        );
        self.swapchain = swapchain;
        self.new_viewport_uniforms = Some(get_new_viewport_uniforms(new_size));
        true
    }

    fn render(&mut self) {
        if let Some(x) = &mut self.previous_frame_end {
            x.cleanup_finished();
        }
        let (i, future): (usize, SwapchainAcquireFuture<Window>) = loop {
            match vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok((id, suboptimal, future)) => {
                    if !suboptimal {
                        break (id, future);
                    }
                }
                Err(AcquireError::OutOfDate) => {}
                Err(x) => panic!("{:?}", x),
            }
            self.resize_loop(self.surface.window().inner_size());
        };

        let data = self.new_data.lock().unwrap();
        self.positions_length = self.engine.run(&*data, &mut self.positions);
        std::mem::drop(data);
        self.pointers.update(self.positions, self.positions_length);
        let mut instances = [Instance::default(); 10];
        for ((i, x, y), instance) in self
            .pointers
            .events()
            .iter()
            .enumerate()
            .filter_map(|(i, report)| match report {
                Report::UpDown((x, y)) | Report::Down((x, y)) | Report::Move((x, y)) => {
                    Some((i, x, y))
                }
                _ => None,
            })
            .zip(instances.iter_mut())
        {
            instance.instance_data[0] = *x;
            instance.instance_data[1] = *y;
            instance.instance_data[2] = i as u32;
        }
        self.new_instance_positions = Some(instances);

        let cmd = {
            let mut builder = AutoCommandBufferBuilder::primary_one_time_submit(
                self.engine.device(),
                self.engine.queue().family(),
            )
            .unwrap();
            if let Some(new_viewport_uniforms) = &self.new_viewport_uniforms {
                builder
                    .update_buffer(
                        self.viewport_uniforms.clone(),
                        Box::new(*new_viewport_uniforms),
                    )
                    .unwrap();
            }
            if let Some(new_instance_positions) = &mut self.new_instance_positions {
                builder
                    .update_buffer(
                        self.instance_positions.clone(),
                        Box::new(new_instance_positions.clone()),
                    )
                    .unwrap();
            }
            builder
                .begin_render_pass(
                    self.framebuffers[i].clone(),
                    SubpassContents::Inline,
                    vec![[0.0, 0.0, 1.0, 1.0].into()],
                )
                .unwrap()
                .draw(
                    self.pipeline.clone(),
                    &self.dynamic_state,
                    BufferlessVertices {
                        vertices: 3,
                        instances: 1,
                    },
                    self.descriptor_set.clone(),
                    (),
                    std::iter::empty(),
                )
                .unwrap()
                .draw(
                    self.cursor_pipeline.clone(),
                    &self.dynamic_state,
                    (
                        [self.vertex_positions.clone()],
                        self.instance_positions
                            .clone()
                            .into_buffer_slice()
                            .slice(0..self.positions_length)
                            .unwrap(),
                    ),
                    (),
                    (),
                    std::iter::empty(),
                )
                .unwrap()
                .end_render_pass()
                .unwrap();
            builder.build().unwrap()
        };

        let frame_end = self
            .previous_frame_end
            .take()
            .unwrap_or_else(|| vulkano::sync::now(self.engine.device()).boxed())
            .join(future)
            .then_execute(self.engine.queue(), cmd)
            .unwrap()
            .then_swapchain_present(self.engine.queue(), self.swapchain.clone(), i)
            .then_signal_fence_and_flush();
        self.previous_frame_end = match frame_end {
            Ok(frame_end) => {
                self.new_viewport_uniforms = None;
                self.new_instance_positions = None;
                // Some(frame_end.boxed())
                frame_end.wait(None).unwrap();
                None
            }
            Err(FlushError::OutOfDate) => {
                self.resize(self.surface.window().inner_size());
                None
            }
            Err(x) => panic!("{:?}", x),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let engine = Engine::new(false);
    let mut state = State::new(engine, &event_loop);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { window_id, event } => {
                assert_eq!(window_id, state.surface.window().id());
                match event {
                    WindowEvent::Resized(size) => state.resize_loop(size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize_loop(*new_inner_size)
                    }
                    WindowEvent::CloseRequested => {
                        state.cleanup();
                        *control_flow = ControlFlow::Exit
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.state == ElementState::Released {
                            match input.virtual_keycode {
                                Some(VirtualKeyCode::R) => {
                                    println!("Reset");
                                    state.reset_sensor.store(true, Ordering::Release);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) => {
                assert_eq!(window_id, state.surface.window().id());
                state.render();
            }
            Event::MainEventsCleared => {
                state.surface.window().request_redraw();
            }
            _ => {}
        }
    });
}
