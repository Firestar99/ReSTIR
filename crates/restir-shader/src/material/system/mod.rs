use crate::visibility::scene::{VisiScene, VisiTriangle};
use glam::Vec4;
use rust_gpu_bindless_shaders::buffer_content::{BufferContent, BufferStruct};
use rust_gpu_bindless_shaders::descriptor::{Buffer, Descriptors, StrongDesc};

pub mod image_shader;

pub trait MaterialEvalFn<T: BufferStruct, M: BufferContent + ?Sized>:
	FnOnce(&T, &mut Descriptors<'_>, VisiScene, VisiTriangle, StrongDesc<Buffer<M>>) -> Vec4
{
}

impl<T: BufferStruct, M: BufferContent + ?Sized, I> MaterialEvalFn<T, M> for I where
	I: FnOnce(&T, &mut Descriptors<'_>, VisiScene, VisiTriangle, StrongDesc<Buffer<M>>) -> Vec4
{
}

#[macro_export]
macro_rules! material_shader {
	($name:ident, $param:ty, $model:ty, $eval:ident) => {
		pub mod $name {
			use super::*;
			$crate::material_shader_image!(image, $param, $model, $eval);
		}
	};
}
