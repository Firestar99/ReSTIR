use crate::camera::Camera;
use crate::utils::affine_transform::AffineTransform;
use crate::visibility::id::{InstanceId, TriangleId};
use core::ops::{Deref, DerefMut};
use glam::Vec3;
use rust_gpu_bindless_macros::BufferStruct;
use rust_gpu_bindless_shaders::descriptor::{Buffer, Descriptors, StrongDesc};

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct VisiScene {
	pub instances: StrongDesc<Buffer<[VisiInstance]>>,
	pub camera: Camera,
}

impl VisiScene {
	pub fn load_instance(&self, descriptors: &Descriptors, instance_id: InstanceId) -> VisiInstance {
		self.instances.access(descriptors).load(instance_id.to_usize())
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct VisiInstance {
	pub model: StrongDesc<Buffer<VisiModel>>,
	pub info: VisiInstanceInfo,
}

impl Deref for VisiInstance {
	type Target = VisiInstanceInfo;

	fn deref(&self) -> &Self::Target {
		&self.info
	}
}

impl DerefMut for VisiInstance {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.info
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct VisiInstanceInfo {
	pub world_from_local: AffineTransform,
}

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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VisiTriangle {
	pub indices: VisiIndices,
	pub vertices: [VisiVertex; 3],
}

impl VisiModel {
	pub fn load_triangle(&self, descriptors: &Descriptors, triangle_id: TriangleId) -> VisiTriangle {
		let indices = self.triangles.access(descriptors).load(triangle_id.to_usize());
		let vertices = [
			self.load_vertex(descriptors, indices[0]),
			self.load_vertex(descriptors, indices[1]),
			self.load_vertex(descriptors, indices[2]),
		];
		VisiTriangle { indices, vertices }
	}

	pub fn load_vertex(&self, descriptors: &Descriptors, vertex_id: u32) -> VisiVertex {
		self.vertices.access(descriptors).load(vertex_id as usize)
	}
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct VisiVertex(pub Vec3);
