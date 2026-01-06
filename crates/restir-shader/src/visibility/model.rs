use crate::visibility::id::TriangleId;
use core::ops::{Deref, DerefMut};
use glam::Vec3;
use rust_gpu_bindless_macros::BufferStruct;
use rust_gpu_bindless_shaders::descriptor::dyn_buffer::DynBuffer;
use rust_gpu_bindless_shaders::descriptor::{Buffer, Descriptors, Strong, StrongDesc};

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct VisiModel {
	/// The triangles of this model
	pub triangles: StrongDesc<Buffer<[VisiIndices]>>,
	/// The vertices of this model
	pub vertices: StrongDesc<Buffer<[VisiVertex]>>,
	// /// A reference to a buffer containing material information. The type contained within the buffer is unknown, and
	// /// the material evaluation shader is expected to transmute the type to the one it expects.
	// pub dyn_material_model: DynBuffer<Strong>,
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
