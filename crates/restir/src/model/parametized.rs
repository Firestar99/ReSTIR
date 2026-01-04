use crate::model::VisiCpuModel;
use glam::{Affine3A, Vec3};
use restir_shader::visibility::model::{VisiIndices, VisiVertex};
use rust_gpu_bindless::descriptor::Bindless;

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
		.into_iter()
		.map(|pos| VisiVertex(transform.transform_point3(Vec3::from_array(*pos))));
	let indices = indices.as_chunks::<3>().0.into_iter().map(|i| VisiIndices(*i));
	VisiCpuModel::new(bindless, vertices, indices)
}
