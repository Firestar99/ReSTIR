use crate::material::light::radiance::Radiance;
use glam::Vec3;
use rust_gpu_bindless_macros::{BufferStruct, assert_transfer_size};

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct DirectionalLight {
	pub direction: Vec3,
	pub color: Radiance,
}
assert_transfer_size!(DirectionalLight, 6 * 4);

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct PointLight {
	pub position: Vec3,
	pub color: Radiance,
}
assert_transfer_size!(PointLight, 6 * 4);

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct AmbientLight {
	pub color: Radiance,
}
assert_transfer_size!(AmbientLight, 3 * 4);

pub trait AnalyticalLightMaterialEval {
	fn eval_directional(&self, light: DirectionalLight) -> Radiance;
	fn eval_point(&self, light: PointLight) -> Radiance;
	fn eval_ambient(&self, light: AmbientLight) -> Radiance;
}
