use crate::visibility::scene::VisiTriangle;
use glam::Vec4;
use rust_gpu_bindless_shaders::buffer_content::BufferStruct;
use rust_gpu_bindless_shaders::descriptor::Descriptors;

pub mod image_shader;

pub trait MaterialEvalFn<T: BufferStruct>: FnOnce(&T, &mut Descriptors<'_>, VisiTriangle) -> Vec4 {}

impl<T: BufferStruct, I> MaterialEvalFn<T> for I where I: FnOnce(&T, &mut Descriptors<'_>, VisiTriangle) -> Vec4 {}

#[macro_export]
macro_rules! material_shader {
	($name:ident, $param:ty, $eval:ident) => {
		pub mod $name {
			use super::*;
			$crate::material_shader_image!(image, $param, $eval);
		}
	};
}
