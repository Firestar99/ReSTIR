use crate::material::pbr::PBR_MODEL_BUFFER_TYPE;
use crate::model::gltf::image::{single_pixel_image, upload_image};
use crate::model::gltf::{Gltf, GltfImageError, Scheme};
use crate::model::{VisiCpuModel, VisiModelNode};
use anyhow::{Context, anyhow};
use futures::future::try_join_all;
use glam::{Affine3A, Vec2, Vec3, Vec4};
use gltf::image::Source;
use gltf::mesh::Mode;
use rayon::prelude::*;
use restir_shader::material::pbr::model::{PbrMaterial, PbrModel, PbrVertex};
use restir_shader::visibility::model::{VisiIndices, VisiVertex};
use restir_shader::visibility::scene::VisiInstanceInfo;
use rust_gpu_bindless::descriptor::{Bindless, BindlessBufferCreateInfo, BindlessBufferUsage, RCDescExt};
use rust_gpu_bindless_shaders::descriptor::dyn_buffer::DynBuffer;
use std::path::Path;
use std::sync::Arc;

pub async fn load_gltf(bindless: &Bindless, path: &Path, transform: Affine3A) -> anyhow::Result<VisiModelNode> {
	let gltf = Gltf::open(path)?;
	let scene = gltf.default_scene().context("gltf no default scene")?;
	let transforms = gltf.absolute_node_transformations(&scene, transform);

	let images = gltf.images().collect::<Vec<_>>();
	let images = images
		.into_par_iter()
		.map(|image| {
			let scheme = match image.source() {
				Source::View { view, .. } => {
					let buffer = gltf.buffer(view.buffer()).ok_or(GltfImageError::MissingBuffer)?;
					Scheme::Slice(
						buffer
							.get(view.offset()..(view.offset() + view.length()))
							.ok_or(GltfImageError::BufferViewOutOfBounds)?,
					)
				}
				Source::Uri { uri, .. } => Scheme::parse(uri).ok_or(GltfImageError::UnsupportedUri)?,
			};

			let data = image::load_from_memory(&scheme.read(gltf.base())?)?;
			Ok(upload_image(
				bindless,
				&data.into_rgba8(),
				image.name().unwrap_or("unnamed model image"),
			))
		})
		.collect::<anyhow::Result<Vec<_>>>()?;

	let default_white = upload_image(bindless, &single_pixel_image(Vec4::ZERO), "default white");
	let default_normal = upload_image(bindless, &single_pixel_image(Vec4::splat(0.5)), "default white");
	let images = try_join_all(images.into_iter()).await?;
	let default_white = default_white.await?;
	let default_normal = default_normal.await?;

	let materials = gltf
		.materials()
		.map(|m| {
			let base_color = m
				.pbr_metallic_roughness()
				.base_color_texture()
				.and_then(|i| images.get(i.texture().index()))
				.map(|rc| rc.to_strong())
				.unwrap_or_else(|| default_white.to_strong());
			let normal = m
				.normal_texture()
				.and_then(|i| images.get(i.texture().index()))
				.map(|rc| rc.to_strong())
				.unwrap_or_else(|| default_normal.to_strong());
			let occlusion_roughness_metallic = m
				.pbr_metallic_roughness()
				.metallic_roughness_texture()
				.and_then(|i| images.get(i.texture().index()))
				.map(|rc| rc.to_strong())
				.unwrap_or_else(|| default_white.to_strong());
			Ok(PbrMaterial {
				base_color,
				base_color_factor: m.pbr_metallic_roughness().base_color_factor(),
				normal,
				normal_scale: m.normal_texture().map_or(1., |n| n.scale()),
				occlusion_roughness_metallic,
				occlusion_strength: 0.,
				roughness_factor: m.pbr_metallic_roughness().roughness_factor(),
				metallic_factor: m.pbr_metallic_roughness().metallic_factor(),
			})
		})
		.collect::<anyhow::Result<Vec<_>>>()?;

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

				let normals = reader.read_normals().ok_or_else(|| anyhow!("missing normals"))?;
				let tangents = reader.read_tangents().ok_or_else(|| anyhow!("missing tangents"))?;
				let tex_coords = reader
					.read_tex_coords(0)
					.ok_or_else(|| anyhow!("missing tex coords 0"))?
					.into_f32();
				let vertices = normals
					.zip(tangents)
					.zip(tex_coords)
					.map(|((normal, tangent), tex_coord)| PbrVertex {
						normal: Vec3::from_array(normal),
						tangent: Vec4::from_array(tangent),
						tex_coord: Vec2::from_array(tex_coord),
					});
				let vertices = bindless.buffer().alloc_shared_from_iter(
					&BindlessBufferCreateInfo {
						usage: BindlessBufferUsage::STORAGE_BUFFER | BindlessBufferUsage::MAP_WRITE,
						allocation_scheme: Default::default(),
						name: "[PbrVertex]",
					},
					vertices,
				)?;

				let material_index = prim.material().index();
				let material = *material_index.and_then(|i| materials.get(i)).ok_or_else(|| {
					anyhow!(
						"primitive {:?} with bad material index {:?}",
						prim.index(),
						material_index
					)
				})?;
				let pbr_model = bindless.buffer().alloc_shared_from_data(
					&BindlessBufferCreateInfo {
						usage: BindlessBufferUsage::STORAGE_BUFFER | BindlessBufferUsage::MAP_WRITE,
						allocation_scheme: Default::default(),
						name: "PbrModel",
					},
					PbrModel {
						material,
						vertices: vertices.to_strong(),
					},
				)?;
				let dyn_model = DynBuffer::new(*PBR_MODEL_BUFFER_TYPE, pbr_model);
				Ok(Arc::new(VisiCpuModel::new(bindless, positions, indices, &dyn_model)?))
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
