use crate::visibility::scene::VisiCpuScene;
use restir_shader::material::image_shader::{MATERIAL_IMAGE_WG_SIZE, Param};
use rust_gpu_bindless::descriptor::{Bindless, RCDescExt};
use rust_gpu_bindless::pipeline::{BindlessComputePipeline, Recording};
use rust_gpu_bindless_shaders::buffer_content::BufferStruct;
use rust_gpu_bindless_shaders::descriptor::dyn_buffer::BufferType;
use rust_gpu_bindless_shaders::descriptor::{Image, Image2d, Image2dU, MutImage, TransientDesc};
use rust_gpu_bindless_shaders::shader::BindlessShader;
use rust_gpu_bindless_shaders::shader_type::ComputeShader;

pub struct MaterialPipeline<T: BufferStruct, M: BufferStruct> {
	material_buffer_type: BufferType<M>,
	image_pipeline: BindlessComputePipeline<Param<'static, T, M>>,
}

impl<T: BufferStruct, M: BufferStruct> MaterialPipeline<T, M> {
	pub fn new(
		bindless: &Bindless,
		material_buffer_type: BufferType<M>,
		image_shader: &impl BindlessShader<ShaderType = ComputeShader, ParamConstant = Param<'static, T, M>>,
	) -> anyhow::Result<Self> {
		Ok(Self {
			material_buffer_type,
			image_pipeline: bindless.create_compute_pipeline(image_shader)?,
		})
	}

	pub fn dispatch_image(&self, cmd: &mut Recording, info: &DispatchImageInfo, param: T) -> anyhow::Result<()> {
		let size = info.scene.camera.viewport_size;
		cmd.dispatch(
			&self.image_pipeline,
			[
				size.x.div_ceil(MATERIAL_IMAGE_WG_SIZE.x),
				size.y.div_ceil(MATERIAL_IMAGE_WG_SIZE.y),
				1,
			],
			Param {
				scene: info.scene.scene.to_transient(cmd),
				material_buffer_type: self.material_buffer_type,
				packed_vertex_image: info.packed_vertex_image,
				depth_image: info.depth_image,
				output_image: info.output_image,
				inner: param,
			},
		)?;
		Ok(())
	}
}

pub struct DispatchImageInfo<'a> {
	pub scene: VisiCpuScene,
	pub packed_vertex_image: TransientDesc<'a, Image<Image2dU>>,
	pub depth_image: TransientDesc<'a, Image<Image2d>>,
	pub output_image: TransientDesc<'a, MutImage<Image2d>>,
}
