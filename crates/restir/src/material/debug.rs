use crate::material::system::material_pipeline::MaterialPipeline;
use restir_shader::material::debug::DebugSettings;
use rust_gpu_bindless::descriptor::Bindless;
use std::ops::Deref;

pub struct VisiDebugPipeline(pub MaterialPipeline<DebugSettings>);

impl VisiDebugPipeline {
	pub fn new(bindless: &Bindless) -> anyhow::Result<Self> {
		Ok(Self(MaterialPipeline::new(
			bindless,
			crate::shader::material::debug::debug_material::image::new(),
		)?))
	}
}

impl Deref for VisiDebugPipeline {
	type Target = MaterialPipeline<DebugSettings>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
