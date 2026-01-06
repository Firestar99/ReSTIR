use crate::material::system::MaterialEvalParam;
use crate::material_shader;
use crate::utils::view_range::DebugValueRange;
use glam::{Vec3, Vec4};
use num_enum::{FromPrimitive, IntoPrimitive};
use rust_gpu_bindless_macros::BufferStruct;
use rust_gpu_bindless_shaders::buffer_content::BufferStructPlain;
use rust_gpu_bindless_shaders::descriptor::Descriptors;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, BufferStruct)]
pub struct DebugMaterial {
	_dummy: u32,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, FromPrimitive, IntoPrimitive)]
pub enum DebugType {
	None,
	#[default]
	ColorfulIds,
	InstanceId,
	TriangleId,
	Barycentrics,
}

impl DebugType {
	pub const MAX_VALUE: DebugType = DebugType::Barycentrics;
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

material_shader!(debug_material, DebugSettings, DebugMaterial, pbr_eval);

fn pbr_eval(_: &mut Descriptors<'_>, p: MaterialEvalParam<'_, DebugSettings, DebugMaterial>) -> Vec4 {
	let geo = p.tri.geo;
	if geo.is_clear {
		Vec4::ZERO
	} else {
		let debug_settings = p.param;
		let view_range = debug_settings.view_range;
		let instance_id_color = || view_range.clamp((geo.instance_id.to_u32() + 1) as f32);
		let triangle_id_color = || view_range.clamp((geo.triangle_id.to_u32() + 1) as f32);
		let color = match debug_settings.debug_type {
			DebugType::None => Vec3::ZERO,
			DebugType::ColorfulIds => Vec3::from((instance_id_color(), triangle_id_color(), 0.)),
			DebugType::InstanceId => Vec3::from((instance_id_color(), 0., 0.)),
			DebugType::TriangleId => Vec3::from((triangle_id_color(), 0., 0.)),
			DebugType::Barycentrics => p.tri.barycentric.lambda.0,
		};
		Vec4::from((color, debug_settings.debug_mix))
	}
}
