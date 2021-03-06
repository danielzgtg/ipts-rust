use std::sync::Arc;

use vulkano::descriptor::pipeline_layout::PipelineLayout;
use vulkano::device::Device;
use vulkano::pipeline::ComputePipeline;

macro_rules! shader {
    ($id: ident, $path: expr) => {
        mod $id {
            vulkano_shaders::shader! {
                ty: "compute",
                path: $path,
            }
            #[allow(dead_code)]
            const _ENSURE_VULKANO_RECOMPILES_CHANGES: &str = include_str!(concat!("../../", $path));
        }
    }
}

shader!(s00, "src/shaders/0_0_unpack.glsl");
shader!(s01, "src/shaders/0_1_invert.glsl");
shader!(s10, "src/shaders/1_0_hightide.glsl");
shader!(s11, "src/shaders/1_1_lowtide.glsl");
shader!(s12, "src/shaders/1_2_lowpass.glsl");
shader!(s30, "src/shaders/3_0_index.glsl");
shader!(s31a, "src/shaders/3_1_a_connect.glsl");
shader!(s31b, "src/shaders/3_1_b_connect.glsl");
shader!(s32, "src/shaders/3_2_activate.glsl");
shader!(s33, "src/shaders/3_3_deactivate.glsl");
shader!(s34, "src/shaders/3_4_pack.glsl");
shader!(s35, "src/shaders/3_5_mask.glsl");
shader!(s37, "src/shaders/3_7_horiz.glsl");
shader!(s38, "src/shaders/3_8_cursor.glsl");

pub struct Shaders {
    pub s00: s00::Shader,
    pub s01: s01::Shader,
    pub s10: s10::Shader,
    pub s11: s11::Shader,
    pub s12: s12::Shader,
    pub s30: s30::Shader,
    pub s31a: s31a::Shader,
    pub s31b: s31b::Shader,
    pub s32: s32::Shader,
    pub s33: s33::Shader,
    pub s34: s34::Shader,
    pub s35: s35::Shader,
    pub s37: s37::Shader,
    pub s38: s38::Shader,
}

impl Shaders {
    pub fn new(device: &Arc<Device>) -> Shaders {
        Shaders {
            s00: s00::Shader::load(device.clone()).unwrap(),
            s01: s01::Shader::load(device.clone()).unwrap(),
            s10: s10::Shader::load(device.clone()).unwrap(),
            s11: s11::Shader::load(device.clone()).unwrap(),
            s12: s12::Shader::load(device.clone()).unwrap(),
            s30: s30::Shader::load(device.clone()).unwrap(),
            s31a: s31a::Shader::load(device.clone()).unwrap(),
            s31b: s31b::Shader::load(device.clone()).unwrap(),
            s32: s32::Shader::load(device.clone()).unwrap(),
            s33: s33::Shader::load(device.clone()).unwrap(),
            s34: s34::Shader::load(device.clone()).unwrap(),
            s35: s35::Shader::load(device.clone()).unwrap(),
            s37: s37::Shader::load(device.clone()).unwrap(),
            s38: s38::Shader::load(device.clone()).unwrap(),
        }
    }
}

pub struct Pipelines {
    pub s00: Arc<ComputePipeline<PipelineLayout<s00::Layout>>>,
    pub s01: Arc<ComputePipeline<PipelineLayout<s01::Layout>>>,
    pub s10: Arc<ComputePipeline<PipelineLayout<s10::Layout>>>,
    pub s11: Arc<ComputePipeline<PipelineLayout<s11::Layout>>>,
    pub s12: Arc<ComputePipeline<PipelineLayout<s12::Layout>>>,
    pub s30: Arc<ComputePipeline<PipelineLayout<s30::Layout>>>,
    pub s31a: Arc<ComputePipeline<PipelineLayout<s31a::Layout>>>,
    pub s31b: Arc<ComputePipeline<PipelineLayout<s31b::Layout>>>,
    pub s32: Arc<ComputePipeline<PipelineLayout<s32::Layout>>>,
    pub s33: Arc<ComputePipeline<PipelineLayout<s33::Layout>>>,
    pub s34: Arc<ComputePipeline<PipelineLayout<s34::Layout>>>,
    pub s35: Arc<ComputePipeline<PipelineLayout<s35::Layout>>>,
    pub s37: Arc<ComputePipeline<PipelineLayout<s37::Layout>>>,
    pub s38: Arc<ComputePipeline<PipelineLayout<s38::Layout>>>,
}

macro_rules! pipeline_init {
    ($device: expr, $shader: expr) => {
        Arc::new(
            ComputePipeline::new($device.clone(), &$shader.main_entry_point(), &(), None).unwrap()
        )
    };
}

impl Pipelines {
    pub fn new(device: &Arc<Device>, shaders: &Shaders) -> Pipelines {
        Pipelines {
            s00: pipeline_init!(device, shaders.s00),
            s01: pipeline_init!(device, shaders.s01),
            s10: pipeline_init!(device, shaders.s10),
            s11: pipeline_init!(device, shaders.s11),
            s12: pipeline_init!(device, shaders.s12),
            s30: pipeline_init!(device, shaders.s30),
            s31a: pipeline_init!(device, shaders.s31a),
            s31b: pipeline_init!(device, shaders.s31b),
            s32: pipeline_init!(device, shaders.s32),
            s33: pipeline_init!(device, shaders.s33),
            s34: pipeline_init!(device, shaders.s34),
            s35: pipeline_init!(device, shaders.s35),
            s37: pipeline_init!(device, shaders.s37),
            s38: pipeline_init!(device, shaders.s38),
        }
    }
}
