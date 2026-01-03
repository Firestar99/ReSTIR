use crate::utils::view_range::DebugValueRange;
use crate::visibility::id::PackedGeometryId;
use crate::visibility::scene::VisiScene;
use glam::{UVec2, UVec3, UVec4, Vec3, Vec3Swizzles, Vec4};
use num_enum::{FromPrimitive, IntoPrimitive};
use rust_gpu_bindless_macros::{BufferStruct, bindless};
use rust_gpu_bindless_shaders::buffer_content::BufferStructPlain;
use rust_gpu_bindless_shaders::descriptor::{Buffer, Descriptors, Image, Image2d, Image2dU, MutImage, TransientDesc};
use static_assertions::const_assert_eq;

#[repr(u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, FromPrimitive, IntoPrimitive)]
pub enum DebugType {
	None,
	#[default]
	ColorfulIds,
	InstanceId,
	TriangleId,
}

impl DebugType {
	pub const MAX_VALUE: DebugType = DebugType::TriangleId;
	pub const LEN: u32 = Self::MAX_VALUE as u32 + 1;
}

unsafe impl BufferStructPlain for DebugType {
	type Transfer = u32;

	unsafe fn write(self) -> Self::Transfer {
		<u32 as From<Self>>::from(self)
	}

	unsafe fn read(from: Self::Transfer) -> Self {
		<Self as num_enum::FromPrimitive>::from_primitive(from)
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct DebugSettings {
	pub debug_type: DebugType,
	pub debug_mix: f32,
	pub view_range: DebugValueRange,
}

impl Default for DebugSettings {
	fn default() -> Self {
		Self {
			debug_type: DebugType::default(),
			debug_mix: 1.0,
			view_range: DebugValueRange::default(),
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone, BufferStruct)]
pub struct Param<'a> {
	pub scene: TransientDesc<'a, Buffer<VisiScene>>,
	pub packed_vertex_image: TransientDesc<'a, Image<Image2dU>>,
	pub output_image: TransientDesc<'a, MutImage<Image2d>>,
	pub debug_settings: DebugSettings,
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
			Vec4::ZERO
		} else {
			let geo = packed_geo.unpack();
			let view_range = param.debug_settings.view_range;
			let instance_id_color = || view_range.clamp((geo.instance_id.to_u32() + 1) as f32);
			let triangle_id_color = || view_range.clamp((geo.triangle_id.to_u32() + 1) as f32);
			let color = match param.debug_settings.debug_type {
				DebugType::None => Vec3::ZERO,
				DebugType::ColorfulIds => Vec3::from((instance_id_color(), triangle_id_color(), 0.)),
				DebugType::InstanceId => Vec3::from((instance_id_color(), 0., 0.)),
				DebugType::TriangleId => Vec3::from((triangle_id_color(), 0., 0.)),
			};
			Vec4::from((color, param.debug_settings.debug_mix))
		};
		unsafe {
			param.output_image.access(&descriptors).write(pixel, out_color);
		}
	}
}
