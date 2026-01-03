use crate::camera::Camera;
use crate::utils::affine_transform::AffineTransform;
use crate::visibility::barycentric::BarycentricDeriv;
use crate::visibility::id::{GeometryId, InstanceId, TriangleId};
use core::ops::{Deref, DerefMut};
use glam::{UVec2, Vec3};
use rust_gpu_bindless_macros::BufferStruct;
use rust_gpu_bindless_shaders::descriptor::{Buffer, Descriptors, StrongDesc};

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct VisiScene {
	pub instances: StrongDesc<Buffer<[VisiInstance]>>,
	pub camera: Camera,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VisiTriangle {
	pub instance: VisiInstance,
	pub model: VisiModel,
	pub indices: VisiIndices,
	pub vertices: [VisiVertex; 3],
	pub barycentric: BarycentricDeriv,
}

impl VisiScene {
	pub fn load_instance(&self, descriptors: &Descriptors, instance_id: InstanceId) -> VisiInstance {
		self.instances.access(descriptors).load(instance_id.to_usize())
	}

	pub fn load_triangle(&self, descriptors: &Descriptors, pixel: UVec2, geo: GeometryId) -> VisiTriangle {
		let instance = self.load_instance(descriptors, geo.instance_id);
		let model = instance.model.access(descriptors).load();
		let indices = model.load_indices(descriptors, geo.triangle_id);
		let vertices = [
			model.load_vertex(descriptors, indices[0]),
			model.load_vertex(descriptors, indices[1]),
			model.load_vertex(descriptors, indices[2]),
		];
		let clip_pos_fn = |i: usize| {
			self.camera
				.transform_vertex(instance.world_from_local, vertices[i].0)
				.clip_space
		};
		let clip_pos = [clip_pos_fn(0), clip_pos_fn(1), clip_pos_fn(2)];
		let viewport = self.camera.viewport_size.as_vec2();
		let pixel_ndc = pixel.as_vec2() / viewport * 2. - 1.;
		let barycentric = BarycentricDeriv::calculate_from(clip_pos[0], clip_pos[1], clip_pos[2], pixel_ndc, viewport);
		VisiTriangle {
			instance,
			model,
			indices,
			vertices,
			barycentric,
		}
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
