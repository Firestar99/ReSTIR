use restir_shader::visibility::scene::{Model, TriangleIndices, Vertex};
use rust_gpu_bindless::descriptor::{
	Bindless, BindlessBufferCreateInfo, BindlessBufferUsage, Buffer, RCDesc, RCDescExt,
};

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct CpuModel {
	pub model: RCDesc<Buffer<Model>>,
	pub indices: RCDesc<Buffer<[u32]>>,
}

impl CpuModel {
	pub fn new(
		bindless: &Bindless,
		vertices: impl Iterator<Item = Vertex> + ExactSizeIterator,
		indices: impl Iterator<Item = TriangleIndices> + ExactSizeIterator,
	) -> anyhow::Result<Self> {
		let triangles = bindless.buffer().alloc_shared_from_iter(
			&BindlessBufferCreateInfo {
				usage: BindlessBufferUsage::MAP_WRITE
					| BindlessBufferUsage::STORAGE_BUFFER
					| BindlessBufferUsage::INDEX_BUFFER,
				allocation_scheme: Default::default(),
				name: "triangle indices",
			},
			indices,
		)?;

		let vertices = bindless.buffer().alloc_shared_from_iter(
			&BindlessBufferCreateInfo {
				usage: BindlessBufferUsage::MAP_WRITE | BindlessBufferUsage::STORAGE_BUFFER,
				allocation_scheme: Default::default(),
				name: "vertices",
			},
			vertices,
		)?;

		let model = bindless.buffer().alloc_shared_from_data(
			&BindlessBufferCreateInfo {
				usage: BindlessBufferUsage::MAP_WRITE | BindlessBufferUsage::STORAGE_BUFFER,
				allocation_scheme: Default::default(),
				name: "model",
			},
			Model {
				triangles: triangles.to_strong(),
				vertices: vertices.to_strong(),
			},
		)?;

		// transmute `[TriangleIndices]` -> `[u32]`
		let indices = unsafe { RCDesc::new_inner(triangles.r) };
		Ok(Self { model, indices })
	}
}
