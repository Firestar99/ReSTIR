use crate::camera::{Camera, TransformedPosition};
use crate::utils::affine_transform::AffineTransform;
use crate::visibility::barycentric::BarycentricDeriv;
use crate::visibility::id::{GeometryId, InstanceId};
use crate::visibility::model::{VisiIndices, VisiModel, VisiVertex};
use core::ops::{Deref, DerefMut};
use glam::{UVec2, Vec4};
use rust_gpu_bindless_macros::BufferStruct;
use rust_gpu_bindless_shaders::descriptor::{AliveDescRef, Buffer, Desc, Descriptors, Image, Image2d, StrongDesc};

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct VisiScene {
	pub instances: StrongDesc<Buffer<[VisiInstance]>>,
	pub camera: Camera,
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
#[derive(Copy, Clone, Debug)]
pub struct VisiTriangle {
	/// integer pixel coordinates
	pub pixel: UVec2,
	/// like glsl `frag_coord` but in different coordinate spaces
	pub frag_coord: TransformedPosition,
	pub geo: GeometryId,
	pub instance: VisiInstance,
	pub model: VisiModel,
	/// indices of the triangle
	pub indices: VisiIndices,
	/// vertices of the triangle
	pub vertices: [VisiVertex; 3],
	pub barycentric: BarycentricDeriv,
}

impl VisiScene {
	pub fn load_instance(&self, descriptors: &Descriptors, instance_id: InstanceId) -> VisiInstance {
		self.instances.access(descriptors).load(instance_id.to_usize())
	}

	pub fn load_triangle(
		&self,
		descriptors: &Descriptors,
		depth_image: Desc<impl AliveDescRef, Image<Image2d>>,
		pixel: UVec2,
		geo: GeometryId,
	) -> VisiTriangle {
		let camera = self.camera;
		let instance = self.load_instance(descriptors, geo.instance_id);
		let model = instance.model.access(descriptors).load();
		let indices = model.load_indices(descriptors, geo.triangle_id);
		let vertices = [
			model.load_vertex(descriptors, indices[0]),
			model.load_vertex(descriptors, indices[1]),
			model.load_vertex(descriptors, indices[2]),
		];
		let clip_pos_fn = |i: usize| {
			camera
				.transform_vertex(instance.world_from_local, vertices[i].0)
				.clip_space
		};
		let clip_pos = [clip_pos_fn(0), clip_pos_fn(1), clip_pos_fn(2)];
		let viewport = camera.viewport_size.as_vec2();
		let barycentric = BarycentricDeriv::calculate_from(clip_pos[0], clip_pos[1], clip_pos[2], pixel, viewport);

		let depth: Vec4 = depth_image.access(descriptors).fetch(pixel);
		let frag_coord = camera.reconstruct_from_depth(pixel.as_vec2() / viewport + 0.5, depth.x);
		VisiTriangle {
			pixel,
			frag_coord,
			geo,
			instance,
			model,
			indices,
			vertices,
			barycentric,
		}
	}
}
