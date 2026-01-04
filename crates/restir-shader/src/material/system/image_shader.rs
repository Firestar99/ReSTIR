//! A material shader that is evaluated on an image

use crate::material::system::MaterialEvalFn;
use crate::visibility::id::PackedGeometryId;
use crate::visibility::scene::VisiScene;
use glam::{UVec2, UVec3, UVec4, Vec3Swizzles};
use rust_gpu_bindless_macros::BufferStruct;
use rust_gpu_bindless_shaders::buffer_content::BufferStruct;
use rust_gpu_bindless_shaders::descriptor::{Buffer, Descriptors, Image, Image2d, Image2dU, MutImage, TransientDesc};
use static_assertions::const_assert_eq;

#[repr(C)]
#[derive(Copy, Clone, BufferStruct)]
pub struct Param<'a, T: BufferStruct> {
	pub scene: TransientDesc<'a, Buffer<VisiScene>>,
	pub packed_vertex_image: TransientDesc<'a, Image<Image2dU>>,
	pub output_image: TransientDesc<'a, MutImage<Image2d>>,
	pub inner: T,
}

pub fn material_shader_image_eval<T: BufferStruct, F: MaterialEvalFn<T>>(
	descriptors: &mut Descriptors<'_>,
	param: &Param<'_, T>,
	wg_id: UVec3,
	inv_id: UVec3,
	eval: F,
) {
	let scene = param.scene.access(&*descriptors).load();
	let size = scene.camera.viewport_size;
	let pixel = wg_id.xy() * MATERIAL_IMAGE_WG_SIZE + inv_id.xy();
	let pixel_inbounds = pixel.x < size.x && pixel.y < size.y;
	if pixel_inbounds {
		let packed_geo: UVec4 = param.packed_vertex_image.access(&*descriptors).fetch_with_lod(pixel, 0);
		let geo = PackedGeometryId::from_u32(packed_geo.x).unpack();
		let tri = scene.load_triangle(&*descriptors, pixel, geo);
		let out_color = eval(&param.inner, &mut *descriptors, scene, tri);
		unsafe {
			param.output_image.access(&*descriptors).write(pixel, out_color);
		}
	}
}

pub const MATERIAL_IMAGE_WG_SIZE: UVec2 = UVec2::new(8, 8);

const_assert_eq!(MATERIAL_IMAGE_WG_SIZE.x, 8);
const_assert_eq!(MATERIAL_IMAGE_WG_SIZE.y, 8);
#[macro_export]
macro_rules! material_shader_image {
	($name:ident, $param:ty, $eval:ident) => {
		#[rust_gpu_bindless_macros::bindless(compute(threads(8, 8)))]
		pub fn $name(
			#[bindless(descriptors)] mut descriptors: rust_gpu_bindless_shaders::descriptor::Descriptors<'_>,
			#[bindless(param)] param: &$crate::material::system::image_shader::Param<'static, $param>,
			#[spirv(workgroup_id)] wg_id: glam::UVec3,
			#[spirv(local_invocation_id)] inv_id: glam::UVec3,
		) {
			$crate::material::system::image_shader::material_shader_image_eval(
				&mut descriptors,
				param,
				wg_id,
				inv_id,
				$eval,
			)
		}
	};
}
