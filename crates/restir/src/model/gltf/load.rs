use crate::material::debug::DEBUG_MATERIAL_BUFFER_TYPE;
use crate::model::gltf::Gltf;
use crate::model::{VisiCpuModel, VisiModelNode};
use anyhow::Context;
use glam::{Affine3A, Vec3};
use gltf::mesh::Mode;
use restir_shader::material::debug::DebugMaterial;
use restir_shader::visibility::model::{VisiIndices, VisiVertex};
use restir_shader::visibility::scene::VisiInstanceInfo;
use rust_gpu_bindless::descriptor::{Bindless, BindlessBufferCreateInfo, BindlessBufferUsage};
use rust_gpu_bindless_shaders::descriptor::dyn_buffer::DynBuffer;
use std::path::Path;
use std::sync::Arc;

pub fn load_gltf(bindless: &Bindless, path: &Path, transform: Affine3A) -> anyhow::Result<VisiModelNode> {
	let gltf = Gltf::open(path)?;
	let scene = gltf.default_scene().context("gltf no default scene")?;
	let transforms = gltf.absolute_node_transformations(&scene, transform);

	let debug_material = bindless.buffer().alloc_shared_from_data(
		&BindlessBufferCreateInfo {
			usage: BindlessBufferUsage::STORAGE_BUFFER | BindlessBufferUsage::MAP_WRITE,
			allocation_scheme: Default::default(),
			name: "DebugMaterial",
		},
		DebugMaterial::default(),
	)?;
	let debug_material = DynBuffer::new(*DEBUG_MATERIAL_BUFFER_TYPE, debug_material);

	let meshes = gltf
		.meshes()
		.flat_map(|mesh| {
			mesh.primitives().map(|prim| {
				assert_eq!(prim.mode(), Mode::Triangles);
				let reader = prim.reader(|b| gltf.buffer(b));
				let positions = reader
					.read_positions()
					.context("gltf missing vertex positions")?
					.map(|i| VisiVertex(transform.transform_point3(Vec3::from_array(i))));
				let indices = reader
					.read_indices()
					.context("gltf missing indices")?
					.into_u32()
					.collect::<Vec<_>>();
				let indices = indices.as_chunks::<3>().0.iter().map(|i| VisiIndices(*i));
				Ok(Arc::new(VisiCpuModel::new(
					bindless,
					positions,
					indices,
					&debug_material,
				)?))
			})
		})
		.collect::<anyhow::Result<Vec<_>>>()?;

	let models = gltf
		.nodes()
		.filter_map(|n| {
			if let Some(mesh) = n.mesh() {
				let mesh = meshes[mesh.index()].clone();
				let transform = VisiInstanceInfo::new(transforms[n.index()]);
				Some((mesh, transform))
			} else {
				None
			}
		})
		.collect();
	Ok(VisiModelNode { models })
}
