use glam::Vec3;
use restir_shader::material::light::analytical::{AmbientLight, DirectionalLight, PointLight};
use restir_shader::material::light::radiance::Radiance;
use restir_shader::material::light::scene::LightScene;
use rust_gpu_bindless::descriptor::{Bindless, BindlessBufferCreateInfo, BindlessBufferUsage, RCDesc, RCDescExt};
use rust_gpu_bindless_shaders::buffer_content::BufferStruct;
use rust_gpu_bindless_shaders::descriptor::Buffer;
use smallvec::SmallVec;

#[derive(Clone, Default)]
pub struct LightSceneCpu {
	pub ambient: AmbientLight,
	pub directional_lights: SmallVec<[DirectionalLight; 4]>,
	pub point_lights: SmallVec<[PointLight; 4]>,
}

impl LightSceneCpu {
	pub fn upload(&self, bindless: &Bindless) -> anyhow::Result<RCDesc<Buffer<LightScene>>> {
		fn upload_maybe_zero<T: BufferStruct>(
			bindless: &Bindless,
			name: &str,
			default: T,
			data: impl ExactSizeIterator<Item = T>,
		) -> anyhow::Result<RCDesc<Buffer<[T]>>> {
			let ci = BindlessBufferCreateInfo {
				usage: BindlessBufferUsage::STORAGE_BUFFER | BindlessBufferUsage::MAP_WRITE,
				allocation_scheme: Default::default(),
				name,
			};
			if data.len() == 0 {
				Ok(bindless
					.buffer()
					.alloc_shared_from_iter(&ci, std::iter::once(default))?)
			} else {
				Ok(bindless.buffer().alloc_shared_from_iter(&ci, data)?)
			}
		}

		let point_lights = upload_maybe_zero(
			bindless,
			"LightScene.point_lights",
			PointLight {
				color: Radiance::default(),
				position: Vec3::default(),
			},
			self.point_lights.iter().copied(),
		)?;
		let directional_lights = upload_maybe_zero(
			bindless,
			"LightScene.directional_lights",
			DirectionalLight {
				color: Radiance::default(),
				direction: Vec3::default(),
			},
			self.directional_lights.iter().copied(),
		)?;

		Ok(bindless.buffer().alloc_shared_from_data(
			&BindlessBufferCreateInfo {
				usage: BindlessBufferUsage::STORAGE_BUFFER | BindlessBufferUsage::MAP_WRITE,
				allocation_scheme: Default::default(),
				name: "LightScene",
			},
			LightScene {
				ambient: self.ambient,
				directional_lights: directional_lights.to_strong(),
				point_lights: point_lights.to_strong(),
			},
		)?)
	}
}
