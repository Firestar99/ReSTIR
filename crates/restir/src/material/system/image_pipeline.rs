use crate::visibility::scene::VisiCpuScene;
use restir_shader::material::system::image_shader::{MATERIAL_IMAGE_WG_SIZE, Param};
use rust_gpu_bindless::descriptor::{Bindless, Image, Image2d, Image2dU, MutImage, RCDescExt, TransientDesc};
use rust_gpu_bindless::pipeline::{BindlessComputePipeline, Recording};
use rust_gpu_bindless_shaders::buffer_content::BufferStruct;
use rust_gpu_bindless_shaders::shader::BindlessShader;
use rust_gpu_bindless_shaders::shader_type::ComputeShader;

pub struct MaterialImagePipeline<T: BufferStruct> {
	pipeline: BindlessComputePipeline<Param<'static, T>>,
}

impl<T: BufferStruct> MaterialImagePipeline<T> {
	pub fn new(
		bindless: &Bindless,
		shader: &impl BindlessShader<ShaderType = ComputeShader, ParamConstant = Param<'static, T>>,
	) -> anyhow::Result<Self> {
		Ok(Self {
			pipeline: bindless.create_compute_pipeline(shader)?,
		})
	}

	pub fn dispatch(
		&self,
		cmd: &mut Recording,
		scene: VisiCpuScene,
		packed_vertex_image: TransientDesc<Image<Image2dU>>,
		depth_image: TransientDesc<Image<Image2d>>,
		output_image: TransientDesc<MutImage<Image2d>>,
		param: T,
	) -> anyhow::Result<()> {
		let size = scene.camera.viewport_size;
		cmd.dispatch(
			&self.pipeline,
			[
				size.x.div_ceil(MATERIAL_IMAGE_WG_SIZE.x),
				size.y.div_ceil(MATERIAL_IMAGE_WG_SIZE.y),
				1,
			],
			Param {
				scene: scene.scene.to_transient(cmd),
				packed_vertex_image,
				depth_image,
				output_image,
				inner: param,
			},
		)?;
		Ok(())
	}
}
