use crate::material::debug::DEBUG_MATERIAL_BUFFER_TYPE;
use crate::model::VisiCpuModel;
use crate::model::gltf::Gltf;
use anyhow::Context;
use glam::{Affine3A, Vec3};
use gltf::mesh::Mode;
use restir_shader::material::debug::DebugMaterial;
use restir_shader::visibility::model::{VisiIndices, VisiVertex};
use rust_gpu_bindless::descriptor::{Bindless, BindlessBufferCreateInfo, BindlessBufferUsage};
use rust_gpu_bindless_shaders::descriptor::dyn_buffer::DynBuffer;
use std::path::Path;

pub fn load_gltf(bindless: &Bindless, path: &Path, transform: Affine3A) -> anyhow::Result<Vec<VisiCpuModel>> {
	let gltf = Gltf::open(path)?;

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
				VisiCpuModel::new(bindless, positions, indices, &debug_material)
			})
		})
		.collect::<Result<Vec<_>, _>>()?;
	Ok(meshes)
}
