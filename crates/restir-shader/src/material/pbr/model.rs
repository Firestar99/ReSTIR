use crate::visibility::barycentric::{Barycentric, BarycentricInterpolatable};
use crate::visibility::model::VisiIndices;
use glam::{Vec2, Vec3, Vec4};
use rust_gpu_bindless_macros::{BufferStruct, BufferStructPlain, assert_transfer_size};
use rust_gpu_bindless_shaders::descriptor::{Buffer, Desc, DescRef, Descriptors, Image, Image2d, Strong, StrongDesc};

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct PbrModel {
	pub vertices: StrongDesc<Buffer<[PbrVertex]>>,
	pub material: PbrMaterial<Strong>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct PbrMaterial<R: DescRef> {
	pub base_color: Desc<R, Image<Image2d>>,
	pub base_color_factor: [f32; 4],
	pub normal: Desc<R, Image<Image2d>>,
	pub normal_scale: f32,
	pub occlusion_roughness_metallic: Desc<R, Image<Image2d>>,
	pub occlusion_strength: f32,
	pub metallic_factor: f32,
	pub roughness_factor: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStructPlain)]
pub struct PbrVertex {
	pub tangent: Vec4,
	pub normal: Vec3,
	pub tex_coord: Vec2,
}
assert_transfer_size!(PbrVertex, 9 * 4);

impl PbrModel {
	pub fn load_vertices(&self, descriptors: &Descriptors, tri: VisiIndices) -> [PbrVertex; 3] {
		let vertices = self.vertices.access(descriptors);
		let load = |i: usize| vertices.load(tri.0[i] as usize);
		[load(0), load(1), load(2)]
	}
}

impl BarycentricInterpolatable for PbrVertex {
	fn interpolate(bary: Barycentric, attr: [Self; 3]) -> Self {
		Self {
			tangent: Vec4::interpolate(bary, [attr[0].tangent, attr[1].tangent, attr[2].tangent]),
			normal: Vec3::interpolate(bary, [attr[0].normal, attr[1].normal, attr[2].normal]),
			tex_coord: Vec2::interpolate(bary, [attr[0].tex_coord, attr[1].tex_coord, attr[2].tex_coord]),
		}
	}
}
