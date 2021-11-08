use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer};
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice, PhysicalDeviceType};
use vulkano::sync::GpuFuture;

use bind_sets::BindSets;
use buffers::Buffers;
use shaders::Pipelines;
use shaders::Shaders;

mod shaders;
mod buffers;
mod bind_sets;

pub struct Engine {
    vk: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    _shaders: Shaders,
    pipelines: Pipelines,
    buffers: Buffers,
    bind_sets: BindSets,
}

macro_rules! command {
    ($engine: expr, $future: expr, $builder: ident, $inner: tt) => {
        {
            let mut $builder = AutoCommandBufferBuilder::primary_one_time_submit(
                $engine.device.clone(),
                $engine.queue.family(),
            ).unwrap();
            $inner(&mut $builder);
            $builder
                .build().unwrap()
                .execute($engine.queue.clone()).unwrap()
                .then_signal_fence_and_flush().unwrap().wait(None).unwrap();
        }
    };
}

macro_rules! dispatch {
    ($engine: expr, $builder: expr, $($count: expr, $id: ident),+ $(,)?) => {
        $(
            $builder.dispatch(
                $count,
                $engine.pipelines.$id.clone(),
                $engine.bind_sets.$id.clone(),
                (),
                std::iter::empty(),
            ).unwrap();
        )+
    }
}

macro_rules! upload_buf {
    ($engine: expr, $builder: expr, $id: ident, $data: expr) => {
        $builder.update_buffer($engine.buffers.$id.clone(), Box::new($data)).unwrap();
    }
}

macro_rules! zero_buf {
    ($engine: expr, $builder: expr, $id: ident) => {
        $builder.fill_buffer($engine.buffers.$id.clone(), 0u32).unwrap();
    }
}

const HEADLESS_INSTANCE_EXTENSIONS: InstanceExtensions = InstanceExtensions::none();
const INSTANCE_EXTENSIONS: InstanceExtensions = InstanceExtensions {
    khr_surface: true,
    khr_xcb_surface: true,
    ..HEADLESS_INSTANCE_EXTENSIONS
};

const HEADLESS_DEVICE_EXTENSIONS: DeviceExtensions = DeviceExtensions {
    khr_storage_buffer_storage_class: true,
    ..DeviceExtensions::none()
};
const DEVICE_EXTENSIONS: DeviceExtensions = DeviceExtensions {
    khr_swapchain: true,
    ..HEADLESS_DEVICE_EXTENSIONS
};

impl Engine {
    pub fn new(headless: bool) -> Engine {
        let vk = Instance::new(None, &if headless {
            HEADLESS_INSTANCE_EXTENSIONS
        } else {
            INSTANCE_EXTENSIONS
        }, None).unwrap();

        #[inline]
        fn get_only<T>(mut iter: impl Iterator<Item = T>) -> T {
            let result = iter.next().unwrap();
            assert!(iter.next().is_none());
            result
        }

        let physical = get_only(PhysicalDevice::enumerate(&vk)
            // .filter(|x| x.name() == "Intel(R) Iris(R) Plus Graphics (ICL GT2)")
            .filter(|x| x.ty() != PhysicalDeviceType::Cpu)
        );
        let family = get_only(physical.queue_families()
            .filter(|x| x.supports_graphics() && x.supports_compute()));
        let (device, queues) = {
            Device::new(
                physical,
                &Features::none(),
                &if headless { HEADLESS_DEVICE_EXTENSIONS } else { DEVICE_EXTENSIONS },
                [(family, 1.0)].iter().cloned(),
            ).unwrap()
        };
        let queue = get_only(queues);

        let shaders = Shaders::new(&device);
        let pipelines = Pipelines::new(&device, &shaders);
        let buffers = Buffers::new(&device, family);
        let bind_sets = BindSets::new(&pipelines, &buffers);

        Engine {
            vk,
            device,
            queue,
            _shaders: shaders,
            pipelines,
            buffers,
            bind_sets,
        }
    }

    pub fn vk(&self) -> Arc<Instance> {
        self.vk.clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub fn get_parsed_buffer(&self) -> buffers::BufferC {
        self.buffers.c.clone()
    }

    pub fn run(&mut self, data: &[u8; 2816], results: &mut [(u32, u32); 10]) -> usize {
        command!(self, cmd, builder, {
            upload_buf!(self, builder, r, *data);
            dispatch!(self, builder,
                [1, 44, 1], s00,
                [1, 44, 1], s01,
            );
            // TODO Benchmark whether things are better with a local size of 32
            dispatch!(self, builder,
                [1, 44, 1], s30,
                [1, 44, 1], s31a,
                [1, 44, 1], s31b,
                [1, 44, 1], s31a,
                [1, 44, 1], s31b,
                [1, 44, 1], s31a,
                [1, 44, 1], s31b,
                [1, 44, 1], s31a,
                [1, 44, 1], s31b,
                [1, 44, 1], s31a,
                [1, 44, 1], s31b,
                [1, 44, 1], s31a,
                [1, 44, 1], s32,
                [1, 44, 1], s33,
                [16, 1, 1], s34,
            );
        });

        let count;
        let starts = {
            let mut starts = [!0u32; 10];
            let results: Vec<u32> = self.buffers.r.read().unwrap().iter()
                .enumerate().filter(|x| *x.1 != 0).map(|x| x.0 as u32).take(11).collect();
            if results.len() == 11 {
                eprintln!("Too many fingers");
                return 0;
            }
            count = results.len();
            starts[..count].copy_from_slice(&results);
            starts
        };

        command!(self, cmd, builder, {
            upload_buf!(self, builder, i, starts);
            dispatch!(self, builder, [1, 44, 1], s35);
            zero_buf!(self, builder, t);
            dispatch!(self, builder,
                [1, 1, 1], s37,
                [1, 1, 1], s38,
            );
        });

        for (i, packed) in self.buffers.p.read().unwrap().iter().enumerate() {
            let x = packed >> 16;
            let y = packed & 0xFFFF;
            results[i] = (x, y);
        }

        count
    }
}
