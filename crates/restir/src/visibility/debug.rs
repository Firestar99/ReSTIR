use crate::visibility::scene::VisiCpuScene;
use restir_shader::visibility::debug::{DEBUG_VISI_WG_SIZE, DebugSettings, Param};
use rust_gpu_bindless::descriptor::{Bindless, Image, Image2d, Image2dU, MutImage, RCDescExt, TransientDesc};
use rust_gpu_bindless::pipeline::{BindlessComputePipeline, Recording};

pub struct VisiDebugPipeline {
	pipeline: BindlessComputePipeline<Param<'static>>,
}

impl VisiDebugPipeline {
	pub fn new(bindless: &Bindless) -> anyhow::Result<Self> {
		Ok(Self {
			pipeline: bindless.create_compute_pipeline(crate::shader::visibility::debug::debug_visi_comp::new())?,
		})
	}

	pub fn dispatch(
		&self,
		cmd: &mut Recording,
		scene: VisiCpuScene,
		debug_settings: DebugSettings,
		packed_vertex_image: TransientDesc<Image<Image2dU>>,
		output_image: TransientDesc<MutImage<Image2d>>,
	) -> anyhow::Result<()> {
		let size = scene.camera.viewport_size;
		cmd.dispatch(
			&self.pipeline,
			[
				size.x.div_ceil(DEBUG_VISI_WG_SIZE.x),
				size.y.div_ceil(DEBUG_VISI_WG_SIZE.y),
				1,
			],
			Param {
				scene: scene.scene.to_transient(cmd),
				packed_vertex_image,
				output_image,
				debug_settings,
			},
		)?;
		Ok(())
	}
}
