use crate::material::pass::MaterialEval;
use crate::material::pipeline::{DispatchImageInfo, MaterialPipeline};
use parking_lot::Mutex;
use restir_shader::material::light::scene::LightScene;
use restir_shader::material::pbr::model::PbrModel;
use restir_shader::material::pbr::shader::PbrMaterialParam;
use rust_gpu_bindless::descriptor::dyn_buffer::register_dyn_buffer_type;
use rust_gpu_bindless::descriptor::{
	AddressMode, Bindless, BindlessSamplerCreateInfo, Filter, RCDesc, RCDescExt, Sampler,
};
use rust_gpu_bindless::pipeline::Recording;
use rust_gpu_bindless_shaders::descriptor::dyn_buffer::BufferType;
use rust_gpu_bindless_shaders::descriptor::Buffer;
use std::sync::{Arc, LazyLock};

pub static PBR_MODEL_BUFFER_TYPE: LazyLock<BufferType<PbrModel>> = LazyLock::new(|| register_dyn_buffer_type());

pub struct VisiPbrPipeline {
	pub pipeline: MaterialPipeline<PbrMaterialParam<'static>, PbrModel>,
	sampler: RCDesc<Sampler>,
	light_scene: Mutex<Option<RCDesc<Buffer<LightScene>>>>,
}

impl VisiPbrPipeline {
	pub fn new(bindless: &Bindless) -> anyhow::Result<Arc<Self>> {
		Ok(Arc::new(Self {
			pipeline: MaterialPipeline::new(
				bindless,
				*PBR_MODEL_BUFFER_TYPE,
				crate::shader::material::pbr::shader::pbr_eval::image::new(),
			)?,
			sampler: bindless.sampler().alloc(&BindlessSamplerCreateInfo {
				mag_filter: Filter::Linear,
				min_filter: Filter::Linear,
				mipmap_mode: Filter::Linear,
				address_mode_u: AddressMode::Repeat,
				address_mode_v: AddressMode::Repeat,
				address_mode_w: AddressMode::Repeat,
				..BindlessSamplerCreateInfo::default()
			})?,
			light_scene: Mutex::new(None),
		}))
	}
}

impl MaterialEval for VisiPbrPipeline {
	fn dispatch_image(&self, cmd: &mut Recording, info: &DispatchImageInfo) -> anyhow::Result<()> {
		self.pipeline.dispatch_image(
			cmd,
			info,
			PbrMaterialParam {
				sampler: self.sampler.to_transient(cmd),
				light_scene: self.light_scene.lock().as_ref().unwrap().to_transient(cmd),
			},
		)
	}
}
