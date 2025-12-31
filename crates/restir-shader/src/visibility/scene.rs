use crate::camera::Camera;
use crate::utils::affine_transform::AffineTransform;
use crate::visibility::id::{InstanceId, TriangleId};
use core::ops::{Deref, DerefMut};
use glam::Vec3;
use rust_gpu_bindless_macros::BufferStruct;
use rust_gpu_bindless_shaders::descriptor::{Buffer, Descriptors, StrongDesc};

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct Scene {
	pub instances: StrongDesc<Buffer<[Instance]>>,
	pub camera: Camera,
}

impl Scene {
	pub fn load_instance(&self, descriptors: &Descriptors, instance_id: InstanceId) -> Instance {
		self.instances.access(descriptors).load(instance_id.to_usize())
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct Instance {
	pub model: StrongDesc<Buffer<Model>>,
	pub info: InstanceInfo,
}

impl Deref for Instance {
	type Target = InstanceInfo;

	fn deref(&self) -> &Self::Target {
		&self.info
	}
}

impl DerefMut for Instance {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.info
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct InstanceInfo {
	pub world_from_local: AffineTransform,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct Model {
	pub triangles: StrongDesc<Buffer<[TriangleIndices]>>,
	pub vertices: StrongDesc<Buffer<[Vertex]>>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct TriangleIndices(pub [u32; 3]);

impl Deref for TriangleIndices {
	type Target = [u32; 3];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for TriangleIndices {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Triangle {
	pub indices: TriangleIndices,
	pub vertices: [Vertex; 3],
}

impl Model {
	pub fn load_triangle(&self, descriptors: &Descriptors, triangle_id: TriangleId) -> Triangle {
		let indices = self.triangles.access(descriptors).load(triangle_id.to_usize());
		let vertices = [
			self.load_vertex(descriptors, indices[0]),
			self.load_vertex(descriptors, indices[1]),
			self.load_vertex(descriptors, indices[2]),
		];
		Triangle { indices, vertices }
	}

	pub fn load_vertex(&self, descriptors: &Descriptors, vertex_id: u32) -> Vertex {
		self.vertices.access(descriptors).load(vertex_id as usize)
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct Vertex {
	pub position: Vec3,
}
