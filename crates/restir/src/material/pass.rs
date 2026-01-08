use crate::material::debug::VisiDebugPipeline;
use crate::material::pipeline::DispatchImageInfo;
use rust_gpu_bindless::descriptor::Bindless;
use rust_gpu_bindless::pipeline::Recording;
use std::sync::Arc;

pub trait MaterialEval {
	fn dispatch_image(&self, cmd: &mut Recording, info: &DispatchImageInfo) -> anyhow::Result<()>;
}

pub struct MaterialPass {
	pub debug: VisiDebugPipeline,
	materials: Vec<Arc<dyn MaterialEval>>,
}

impl MaterialPass {
	pub fn new(bindless: &Bindless) -> anyhow::Result<Self> {
		let debug = VisiDebugPipeline::new(bindless)?;
		Ok(Self {
			materials: Vec::from([debug.0.clone()]),
			debug,
		})
	}

	pub fn add_material(&mut self, material: Arc<dyn MaterialEval>) {
		self.materials.push(material)
	}
}

impl MaterialEval for MaterialPass {
	fn dispatch_image(&self, cmd: &mut Recording, info: &DispatchImageInfo) -> anyhow::Result<()> {
		for mat in &self.materials {
			mat.dispatch_image(cmd, info)?;
		}
		Ok(())
	}
}
