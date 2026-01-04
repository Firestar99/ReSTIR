use crate::material::light::analytical::{AmbientLight, AnalyticalLightMaterialEval, DirectionalLight, PointLight};
use crate::material::light::radiance::Radiance;
use rust_gpu_bindless_macros::BufferStruct;
use rust_gpu_bindless_shaders::descriptor::{Buffer, Descriptors, StrongDesc};

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct LightScene {
	pub ambient: AmbientLight,
	pub directional_lights: StrongDesc<Buffer<[DirectionalLight]>>,
	pub point_lights: StrongDesc<Buffer<[PointLight]>>,
}

impl LightScene {
	pub fn eval<M: AnalyticalLightMaterialEval>(&self, descriptors: &Descriptors, material: M) -> Radiance {
		let mut out = material.eval_ambient(self.ambient);

		let directional_lights = self.directional_lights.access(descriptors);
		for i in 0..directional_lights.len() {
			out += material.eval_directional(directional_lights.load(i));
		}

		let point_lights = self.point_lights.access(descriptors);
		for i in 0..point_lights.len() {
			out += material.eval_point(point_lights.load(i));
		}
		out
	}
}
