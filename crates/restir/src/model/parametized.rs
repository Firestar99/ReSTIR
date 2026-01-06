use crate::material::debug::DEBUG_MATERIAL_BUFFER_TYPE;
use crate::model::VisiCpuModel;
use glam::{Affine3A, Vec3};
use restir_shader::material::debug::DebugMaterial;
use restir_shader::visibility::model::{VisiIndices, VisiVertex};
use rust_gpu_bindless::descriptor::{Bindless, BindlessBufferCreateInfo, BindlessBufferUsage};
use rust_gpu_bindless_shaders::descriptor::dyn_buffer::DynBuffer;

pub fn cube(bindless: &Bindless, transform: Affine3A) -> anyhow::Result<VisiCpuModel> {
	// from https://en.wikibooks.org/wiki/OpenGL_Programming/Modern_OpenGL_Tutorial_05
	#[rustfmt::skip]
    let vertices = [
        // front
        -1.0, -1.0,  1.0,
        1.0, -1.0,  1.0,
        1.0,  1.0,  1.0,
        -1.0,  1.0,  1.0,
        // back
        -1.0, -1.0, -1.0,
        1.0, -1.0, -1.0,
        1.0,  1.0, -1.0,
        -1.0,  1.0, -1.0
    ];
	#[rustfmt::skip]
    let indices = [
        // front
        0, 1, 2,
        2, 3, 0,
        // right
        1, 5, 6,
        6, 2, 1,
        // back
        7, 6, 5,
        5, 4, 7,
        // left
        4, 0, 3,
        3, 7, 4,
        // bottom
        4, 5, 1,
        1, 0, 4,
        // top
        3, 2, 6,
        6, 7, 3
    ];

	let vertices = vertices
		.as_chunks::<3>()
		.0
		.iter()
		.map(|pos| VisiVertex(transform.transform_point3(Vec3::from_array(*pos))));
	let indices = indices.as_chunks::<3>().0.iter().map(|i| VisiIndices(*i));

	let debug_material = bindless.buffer().alloc_shared_from_data(
		&BindlessBufferCreateInfo {
			usage: BindlessBufferUsage::STORAGE_BUFFER | BindlessBufferUsage::MAP_WRITE,
			allocation_scheme: Default::default(),
			name: "DebugMaterial",
		},
		DebugMaterial::default(),
	)?;
	let debug_material = DynBuffer::new(*DEBUG_MATERIAL_BUFFER_TYPE, debug_material);

	VisiCpuModel::new(bindless, vertices, indices, &debug_material)
}
