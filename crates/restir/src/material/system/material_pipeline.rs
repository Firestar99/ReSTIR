use crate::material::system::image_pipeline::MaterialImagePipeline;
use restir_shader::material::system::image_shader::Param;
use rust_gpu_bindless::descriptor::Bindless;
use rust_gpu_bindless_shaders::buffer_content::BufferStruct;
use rust_gpu_bindless_shaders::shader::BindlessShader;
use rust_gpu_bindless_shaders::shader_type::ComputeShader;

pub struct MaterialPipeline<T: BufferStruct> {
	pub image: MaterialImagePipeline<T>,
}

impl<T: BufferStruct> MaterialPipeline<T> {
	pub fn new(
		bindless: &Bindless,
		image: &impl BindlessShader<ShaderType = ComputeShader, ParamConstant = Param<'static, T>>,
	) -> anyhow::Result<Self> {
		Ok(Self {
			image: MaterialImagePipeline::new(bindless, image)?,
		})
	}
}
