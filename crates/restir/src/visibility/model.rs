use restir_shader::visibility::scene::{Model, TriangleIndices, Vertex};
use rust_gpu_bindless::__private::static_assertions::const_assert_eq;
use rust_gpu_bindless::descriptor::{
	Bindless, BindlessBufferCreateInfo, BindlessBufferUsage, Buffer, DescBufferLenExt, RCDesc, RCDescExt,
};

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct CpuModel {
	pub model: RCDesc<Buffer<Model>>,
	pub indices: RCDesc<Buffer<[u32]>>,
	/// Use this instead of `indices.len()`. Silly len repr in bindless strikes again.
	pub indices_count: u32,
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
				name: "visi model indices",
			},
			indices,
		)?;

		let vertices = bindless.buffer().alloc_shared_from_iter(
			&BindlessBufferCreateInfo {
				usage: BindlessBufferUsage::MAP_WRITE | BindlessBufferUsage::STORAGE_BUFFER,
				allocation_scheme: Default::default(),
				name: "visi model vertices",
			},
			vertices,
		)?;

		let model = bindless.buffer().alloc_shared_from_data(
			&BindlessBufferCreateInfo {
				usage: BindlessBufferUsage::MAP_WRITE | BindlessBufferUsage::STORAGE_BUFFER,
				allocation_scheme: Default::default(),
				name: "visi model",
			},
			Model {
				triangles: triangles.to_strong(),
				vertices: vertices.to_strong(),
			},
		)?;

		// transmute `[TriangleIndices]` -> `[u32]`
		let indices = unsafe { RCDesc::new_inner(triangles.r) };
		const_assert_eq!(3, size_of::<TriangleIndices>() / size_of::<u32>());
		let indices_count = indices.len() as u32 * 3;
		Ok(Self {
			model,
			indices,
			indices_count,
		})
	}
}
