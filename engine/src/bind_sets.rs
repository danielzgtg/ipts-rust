use std::sync::Arc;

use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet, StdDescriptorPoolAlloc, PersistentDescriptorSetBuf};

use crate::buffers::*;
use crate::shaders::Pipelines;

macro_rules! bind_set {
    ($pipeline: expr, $($buffer: expr),*) => {
        Arc::new({
            PersistentDescriptorSet::start(
                $pipeline.layout().descriptor_set_layout(0).unwrap().clone()
            )$(
                .add_buffer($buffer.clone()).unwrap()
            )*.build().unwrap()
        })
    };
}

type BindSet2<A, B> = Arc<PersistentDescriptorSet<
    (((),
      PersistentDescriptorSetBuf<A>),
     PersistentDescriptorSetBuf<B>),
    StdDescriptorPoolAlloc>>;
type BindSet3<A, B, C> = Arc<PersistentDescriptorSet<
    ((((),
       PersistentDescriptorSetBuf<A>),
      PersistentDescriptorSetBuf<B>),
     PersistentDescriptorSetBuf<C>),
    StdDescriptorPoolAlloc>>;

pub struct BindSets {
    pub s00: BindSet2<BufferR, BufferA>,
    pub s01: BindSet2<BufferA, BufferC>,
    pub s30: BindSet2<BufferC, BufferB>,
    pub s31a: BindSet2<BufferB, BufferA>,
    pub s31b: BindSet2<BufferA, BufferB>,
    pub s32: BindSet2<BufferA, BufferB>,
    pub s33: BindSet2<BufferA, BufferB>,
    pub s34: BindSet2<BufferB, BufferR>,
    pub s35: BindSet3<BufferI, BufferA, BufferB>,
    pub s37: BindSet3<BufferA, BufferC, BufferT>,
    pub s38: BindSet2<BufferT, BufferP>,
}

impl BindSets {
    pub fn new(pipelines: &Pipelines, buffers: &Buffers) -> BindSets {
        BindSets {
            s00: bind_set!(pipelines.s00, buffers.r, buffers.a),
            s01: bind_set!(pipelines.s01, buffers.a, buffers.c),
            s30: bind_set!(pipelines.s30, buffers.c, buffers.b),
            s31a: bind_set!(pipelines.s31a, buffers.b, buffers.a),
            s31b: bind_set!(pipelines.s31b, buffers.a, buffers.b),
            s32: bind_set!(pipelines.s32, buffers.a, buffers.b),
            s33: bind_set!(pipelines.s33, buffers.a, buffers.b),
            s34: bind_set!(pipelines.s34, buffers.b, buffers.r),
            s35: bind_set!(pipelines.s35, buffers.i, buffers.a, buffers.b),
            s37: bind_set!(pipelines.s37, buffers.a, buffers.c, buffers.t),
            s38: bind_set!(pipelines.s38, buffers.t, buffers.p),
        }
    }
}


