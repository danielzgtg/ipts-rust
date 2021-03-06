use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBuffer};
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};
use vulkano::sync::GpuFuture;

use bind_sets::BindSets;
use buffers::Buffers;
use shaders::Pipelines;
use shaders::Shaders;

mod shaders;
mod buffers;
mod bind_sets;

pub struct Engine {
    device: Arc<Device>,
    queue: Arc<Queue>,
    _shaders: Shaders,
    pipelines: Pipelines,
    buffers: Buffers,
    bind_sets: BindSets,
}

macro_rules! command {
    ($engine: expr, $builder: ident, $inner: tt) => {
        {
            let mut $builder = AutoCommandBufferBuilder::primary_one_time_submit(
                $engine.device.clone(),
                $engine.queue.family(),
            ).unwrap();
            $inner(&mut $builder);
            $builder
        }.build().unwrap()
    };
}

macro_rules! stage {
    ($engine: expr, $builder: expr, $($count: expr, $id: ident),* $(,)?) => {
        $(
            $builder.dispatch(
                $count,
                $engine.pipelines.$id.clone(),
                $engine.bind_sets.$id.clone(),
                (),
            ).unwrap();
        )*
    }
}

impl Engine {
    pub fn new() -> Engine {
        let vk = Instance::new(None, &InstanceExtensions::none(), None).unwrap();

        #[inline]
        fn get_only<T>(mut iter: impl Iterator<Item = T>) -> T {
            let result = iter.next().unwrap();
            assert!(iter.next().is_none());
            result
        }

        let physical = get_only(PhysicalDevice::enumerate(&vk)
            .filter(|x| x.name() == "Intel(R) Iris(R) Plus Graphics (ICL GT2)"));
        let family = get_only(physical.queue_families());
        let (device, queues) = {
            Device::new(
                physical,
                &Features::none(),
                &DeviceExtensions {
                    khr_storage_buffer_storage_class: true,
                    ..DeviceExtensions::none()
                },
                [(family, 1.0)].iter().cloned(),
            ).unwrap()
        };
        let queue = get_only(queues);

        let shaders = Shaders::new(&device);
        let pipelines = Pipelines::new(&device, &shaders);
        let buffers = Buffers::new(&device, family);
        let bind_sets = BindSets::new(&pipelines, &buffers);

        Engine {
            device,
            queue,
            _shaders: shaders,
            pipelines,
            buffers,
            bind_sets,
        }
    }

    pub fn run(&mut self, data: &[u8; 2816], results: &mut [(u32, u32); 10]) -> usize {
        command!(self, builder, {
            builder.update_buffer(self.buffers.r.clone(), *data).unwrap();
            stage!(
                self, builder,
                [1, 44, 1], s00,
                [1, 44, 1], s01,
                // [1, 44, 1], s10,
                // [1, 44, 1], s11,
                // [1, 44, 1], s12,
            );
            // Disabled the lowpass because it doesn't like when I press my screen lightly
            // Also, the hardware already seems to be doing something similar
            builder.copy_buffer(self.buffers.c.clone(), self.buffers.b.clone()).unwrap();
            stage!(
                self, builder,
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
        })
            .execute(self.queue.clone()).unwrap()
            .then_signal_fence_and_flush().unwrap()
            .wait(None).unwrap();

        let count;
        let starts = {
            let mut starts = [0u32; 10];
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

        command!(self, builder, {
            builder.update_buffer(self.buffers.i.clone(), starts).unwrap();
            stage!(self, builder, [1, 44, 1], s35);
            builder.fill_buffer(self.buffers.t.clone(), 0u32).unwrap();
            stage!(
                self, builder,
                [1, 10, 1], s37,
                [1, 1, 1], s38,
            );
        })
            .execute( self.queue.clone()).unwrap()
            .then_signal_fence_and_flush().unwrap()
            .wait(None).unwrap();

        for (i, packed) in self.buffers.p.read().unwrap().iter().enumerate() {
            let x = packed >> 16;
            let y = packed & 0xFFFF;
            results[i] = (x, y);
        }

        count
    }
}

