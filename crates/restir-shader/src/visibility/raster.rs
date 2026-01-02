use crate::visibility::id::{InstanceId, PackedGeometryId, TriangleId};
use crate::visibility::scene::{Model, Scene};
use glam::Vec4;
use rust_gpu_bindless_macros::{BufferStruct, bindless};
use rust_gpu_bindless_shaders::descriptor::{Buffer, Descriptors, TransientDesc};

#[derive(Copy, Clone, BufferStruct)]
pub struct Param<'a> {
	pub scene: TransientDesc<'a, Buffer<Scene>>,
	/// model must be the same as `scene.load_instance(..., instance_index).model`
	pub model: TransientDesc<'a, Buffer<Model>>,
}

#[bindless(vertex())]
pub fn visibility_vert(
	#[bindless(descriptors)] descriptors: Descriptors<'_>,
	#[bindless(param)] param: &Param<'static>,
	#[spirv(vertex_index)] vertex_id: u32,
	#[spirv(instance_index)] instance_id: u32,
	#[spirv(position)] out_position: &mut Vec4,
	#[spirv(flat)] packed_vertex_id: &mut PackedGeometryId,
) {
	let instance_id = unsafe { InstanceId::new_unchecked(instance_id) };
	let scene = param.scene.access(&descriptors).load();
	let instance = scene.load_instance(&descriptors, instance_id);

	let model = param.model.access(&descriptors).load();
	let vertex = model.load_vertex(&descriptors, vertex_id);

	let vtx_pos = scene
		.camera
		.transform_vertex(instance.world_from_local, vertex.position);
	*out_position = vtx_pos.clip_space;

	let triangle_id = unsafe { TriangleId::new_unchecked(vertex_id % 3) };
	*packed_vertex_id = PackedGeometryId::new(instance_id, triangle_id);
}

#[bindless(fragment())]
pub fn visibility_frag(
	// #[bindless(descriptors)] descriptors: Descriptors<'_>,
	#[bindless(param)] _param: &Param<'static>,
	#[spirv(flat)] vertex_id: u32,
	out_vertex_id: &mut u32,
) {
	*out_vertex_id = vertex_id;
}
