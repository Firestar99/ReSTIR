use crate::visibility::id::PackedGeometryId;
use crate::visibility::scene::Scene;
use glam::{UVec2, UVec3, UVec4, Vec3, Vec3Swizzles, Vec4};
use rust_gpu_bindless_macros::{BufferStruct, bindless};
use rust_gpu_bindless_shaders::descriptor::{Buffer, Descriptors, Image, Image2d, Image2dU, MutImage, TransientDesc};
use static_assertions::const_assert_eq;

#[derive(Copy, Clone, BufferStruct)]
pub struct Param<'a> {
	pub scene: TransientDesc<'a, Buffer<Scene>>,
	pub packed_vertex_image: TransientDesc<'a, Image<Image2dU>>,
	pub output_image: TransientDesc<'a, MutImage<Image2d>>,
	pub instance_max: u32,
}

pub const DEBUG_VISI_WG_SIZE: UVec2 = UVec2::new(8, 8);

const_assert_eq!(DEBUG_VISI_WG_SIZE.x, 8);
const_assert_eq!(DEBUG_VISI_WG_SIZE.y, 8);
#[bindless(compute(threads(8, 8)))]
pub fn debug_visi_comp(
	#[bindless(descriptors)] descriptors: Descriptors<'_>,
	#[bindless(param)] param: &Param<'static>,
	#[spirv(workgroup_id)] wg_id: UVec3,
	#[spirv(local_invocation_id)] inv_id: UVec3,
) {
	let wg_id = wg_id.xy();
	let inv_id = inv_id.xy();
	let pixel = wg_id * DEBUG_VISI_WG_SIZE + inv_id;

	let scene = param.scene.access(&descriptors).load();
	let size = scene.camera.viewport_size;
	let pixel_inbounds = pixel.x < size.x && pixel.y < size.y;
	if pixel_inbounds {
		let packed_geo: UVec4 = param.packed_vertex_image.access(&descriptors).fetch_with_lod(pixel, 0);
		let packed_geo = PackedGeometryId::from_u32(packed_geo.x);

		let out_color = if packed_geo.is_clear() {
			Vec4::new(0., 0.1, 0., 0.)
		} else {
			let geo = packed_geo.unpack();
			let out_color = geo.instance_id.to_u32() as f32 / param.instance_max as f32;
			Vec4::from((out_color, Vec3::ZERO))
		};
		unsafe {
			param.output_image.access(&descriptors).write(pixel, out_color);
		}
	}
}
