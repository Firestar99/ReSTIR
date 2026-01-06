use crate::material::system::material_pipeline::MaterialPipeline;
use restir_shader::material::debug::{DebugMaterial, DebugSettings};
use rust_gpu_bindless::descriptor::Bindless;
use rust_gpu_bindless::descriptor::dyn_buffer::register_dyn_buffer_type;
use rust_gpu_bindless_shaders::descriptor::dyn_buffer::BufferType;
use std::ops::Deref;
use std::sync::LazyLock;

pub const DEBUG_MATERIAL_BUFFER_TYPE: LazyLock<BufferType<DebugMaterial>> =
	LazyLock::new(|| register_dyn_buffer_type());

pub struct VisiDebugPipeline(pub MaterialPipeline<DebugSettings, DebugMaterial>);

impl VisiDebugPipeline {
	pub fn new(bindless: &Bindless) -> anyhow::Result<Self> {
		Ok(Self(MaterialPipeline::new(
			bindless,
			*DEBUG_MATERIAL_BUFFER_TYPE,
			crate::shader::material::debug::debug_material::image::new(),
		)?))
	}
}

impl Deref for VisiDebugPipeline {
	type Target = MaterialPipeline<DebugSettings, DebugMaterial>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
