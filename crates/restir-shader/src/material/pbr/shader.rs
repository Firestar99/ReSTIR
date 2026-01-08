use crate::material::MaterialEvalParam;
use crate::material::light::scene::LightScene;
use crate::material::pbr::eval::SurfaceLocation;
use crate::material::pbr::model::PbrModel;
use crate::material_shader;
use glam::Vec4;
use rust_gpu_bindless_macros::BufferStruct;
use rust_gpu_bindless_shaders::descriptor::{Buffer, Descriptors, TransientDesc};
use spirv_std::Sampler;
use spirv_std::image::ImageWithMethods;
use spirv_std::image::sample_with::grad;

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct MaterialParam<'a> {
	pub sampler: TransientDesc<'a, Sampler>,
	pub light_scene: TransientDesc<'a, Buffer<LightScene>>,
}

material_shader!(pbr_eval, MaterialParam<'static>, PbrModel, pbr_eval);

fn pbr_eval(descriptors: &mut Descriptors<'_>, p: MaterialEvalParam<'_, MaterialParam, PbrModel>) -> Vec4 {
	let descriptors = &*descriptors;
	let tri = p.tri;
	let model = p.material.access(descriptors).load();

	let vtx_tri = model.load_vertices(descriptors, tri.indices);
	let vtx = tri.barycentric.lambda.interpolate(vtx_tri);
	let vtx_ddx = tri.barycentric.ddx.interpolate(vtx_tri);
	let vtx_ddy = tri.barycentric.ddy.interpolate(vtx_tri);

	let loc = SurfaceLocation::new(
		tri.frag_coord.world_space,
		p.scene.camera.view_from_world.translation(),
		vtx.normal,
		vtx.tangent,
	);
	let sampled_material = model.material.sample(descriptors, loc, |image, descriptors| {
		let sampler = p.param.sampler.access(descriptors);
		image
			.access(descriptors)
			.sample_with(sampler, vtx.tex_coord, grad(vtx_ddx.tex_coord, vtx_ddy.tex_coord))
	});

	let light_scene = p.param.light_scene.access(descriptors).load();
	let radiance = light_scene.eval(descriptors, sampled_material);

	Vec4::from((radiance.0, 1.))
}
