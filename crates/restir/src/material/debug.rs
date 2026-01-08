use crate::material::pass::MaterialEval;
use crate::material::pipeline::{DispatchImageInfo, MaterialPipeline};
use parking_lot::Mutex;
use restir_shader::material::debug::{DebugMaterial, DebugSettings};
use rust_gpu_bindless::descriptor::Bindless;
use rust_gpu_bindless::descriptor::dyn_buffer::register_dyn_buffer_type;
use rust_gpu_bindless::pipeline::Recording;
use rust_gpu_bindless_shaders::descriptor::dyn_buffer::BufferType;
use std::sync::{Arc, LazyLock};

pub static DEBUG_MATERIAL_BUFFER_TYPE: LazyLock<BufferType<DebugMaterial>> =
	LazyLock::new(|| register_dyn_buffer_type());

pub struct VisiDebugPipeline {
	pub pipeline: MaterialPipeline<DebugSettings, DebugMaterial>,
	pub state: Mutex<DebugSettings>,
}

impl VisiDebugPipeline {
	pub fn new(bindless: &Bindless) -> anyhow::Result<Arc<Self>> {
		Ok(Arc::new(Self {
			pipeline: MaterialPipeline::new(
				bindless,
				*DEBUG_MATERIAL_BUFFER_TYPE,
				crate::shader::material::debug::debug_material::image::new(),
			)?,
			state: Mutex::new(DebugSettings::default()),
		}))
	}

	pub fn set_state(&self, state: DebugSettings) {
		*self.state.lock() = state;
	}
}

impl MaterialEval for VisiDebugPipeline {
	fn dispatch_image(&self, cmd: &mut Recording, info: &DispatchImageInfo) -> anyhow::Result<()> {
		self.pipeline.dispatch_image(cmd, info, *self.state.lock())
	}
}
