use std::sync::Arc;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, DeviceLocalBuffer};
use vulkano::device::Device;
use vulkano::instance::QueueFamily;
use vulkano::memory::pool::{PotentialDedicatedAllocation, StdMemoryPoolAlloc};

type AllocStuff = PotentialDedicatedAllocation<StdMemoryPoolAlloc>;
type CpuGpuBuffer<T> = Arc<CpuAccessibleBuffer<T, AllocStuff>>;
type GpuBuffer<T> = Arc<DeviceLocalBuffer<T, AllocStuff>>;

pub type BufferR = CpuGpuBuffer<[u8; 2816]>;
pub type BufferA = GpuBuffer<[u32; 2816]>;
pub type BufferB = GpuBuffer<[u32; 2816]>;
pub type BufferC = GpuBuffer<[u32; 2816]>;
pub type BufferI = CpuGpuBuffer<[u32; 10]>;
pub type BufferT = GpuBuffer<[u32; 1280]>;
pub type BufferP = CpuGpuBuffer<[u32; 10]>;

pub struct Buffers {
    pub r: BufferR,
    pub a: BufferA,
    pub b: BufferB,
    pub c: BufferC,
    pub i: BufferI,
    pub t: BufferT,
    pub p: BufferP,
}

macro_rules! cpu_gpu_buffer {
    ($device: expr, $init: expr) => {
        CpuAccessibleBuffer::from_data($device.clone(), BufferUsage {
            transfer_source: true,
            transfer_destination: true,
            storage_buffer: true,
            ..BufferUsage::none()
        }, true, $init).unwrap()
    };
}

macro_rules! gpu_buffer {
    ($device: expr, $family: expr) => {
        DeviceLocalBuffer::new($device.clone(), BufferUsage {
            storage_buffer: true,
            ..BufferUsage::none()
        }, std::iter::once($family)).unwrap()
    };
}

impl Buffers {
    pub fn new(device: &Arc<Device>, family: QueueFamily) -> Buffers {
        Buffers {
            r: cpu_gpu_buffer!(device, [0u8; 2816]),
            a: gpu_buffer!(device, family),
            // b: gpu_buffer!(device, family),
            // c: gpu_buffer!(device, family),
            // Enabled copy because I disabled lowpass
            b: DeviceLocalBuffer::new(device.clone(), BufferUsage {
                transfer_destination: true,
                storage_buffer: true,
                ..BufferUsage::none()
            }, std::iter::once(family)).unwrap(),
            c: DeviceLocalBuffer::new(device.clone(), BufferUsage {
                transfer_source: true,
                storage_buffer: true,
                ..BufferUsage::none()
            }, std::iter::once(family)).unwrap(),
            i: cpu_gpu_buffer!(device, [0u32; 10]),
            t: DeviceLocalBuffer::new(device.clone(), BufferUsage {
                transfer_destination: true,
                storage_buffer: true,
                ..BufferUsage::none()
            }, std::iter::once(family)).unwrap(),
            p: cpu_gpu_buffer!(device, [0u32; 10]),
        }
    }
}
