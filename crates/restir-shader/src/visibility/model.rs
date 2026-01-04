use crate::visibility::id::TriangleId;
use core::ops::{Deref, DerefMut};
use glam::Vec3;
use rust_gpu_bindless_macros::BufferStruct;
use rust_gpu_bindless_shaders::descriptor::{Buffer, Descriptors, StrongDesc};

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct VisiModel {
	pub triangles: StrongDesc<Buffer<[VisiIndices]>>,
	pub vertices: StrongDesc<Buffer<[VisiVertex]>>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct VisiIndices(pub [u32; 3]);

impl Deref for VisiIndices {
	type Target = [u32; 3];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for VisiIndices {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl VisiModel {
	pub fn load_indices(&self, descriptors: &Descriptors, triangle_id: TriangleId) -> VisiIndices {
		self.triangles.access(descriptors).load(triangle_id.to_usize())
	}

	pub fn load_vertex(&self, descriptors: &Descriptors, vertex_id: u32) -> VisiVertex {
		self.vertices.access(descriptors).load(vertex_id as usize)
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct VisiVertex(pub Vec3);
