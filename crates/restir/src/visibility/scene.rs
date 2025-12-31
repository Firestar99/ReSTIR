use crate::visibility::model::CpuModel;
use restir_shader::camera::Camera;
use restir_shader::visibility::id::InstanceId;
use restir_shader::visibility::scene::{Instance, InstanceInfo, Scene};
use rust_gpu_bindless::descriptor::{
	Bindless, BindlessBufferCreateInfo, BindlessBufferUsage, Buffer, RCDesc, RCDescExt,
};
use std::collections::HashMap;

pub struct CpuSceneAccum {
	pub instances: HashMap<CpuModel, Vec<Instance>>,
}

impl CpuSceneAccum {
	pub fn new() -> Self {
		Self {
			instances: HashMap::new(),
		}
	}

	pub fn push(&mut self, model: &CpuModel, instance: InstanceInfo) {
		let instance = Instance {
			model: model.model.to_strong(),
			info: instance,
		};
		self.instances
			.entry(model.clone())
			.or_insert_with(Vec::new)
			.push(instance);
	}

	pub fn finish(self, bindless: &Bindless, camera: Camera) -> anyhow::Result<CpuScene> {
		let mut instance_data = Vec::with_capacity(self.instances.iter().map(|(_, i)| i.len()).sum());
		let draws = self
			.instances
			.into_iter()
			.map(|(model, instances)| {
				let instance_start = instance_data.len() as u32;
				let instance_count = instances.len() as u32;
				// verify no oob in shaders later
				InstanceId::new(instance_start + instance_count)?;
				instance_data.extend(instances.into_iter());
				Ok(CpuDraw {
					model,
					instance_start,
					instance_count,
				})
			})
			.collect::<anyhow::Result<Vec<_>>>()?;
		let instance_total_count = instance_data.len() as u32;

		let instance_buffer = bindless.buffer().alloc_shared_from_iter(
			&BindlessBufferCreateInfo {
				usage: BindlessBufferUsage::MAP_WRITE | BindlessBufferUsage::STORAGE_BUFFER,
				allocation_scheme: Default::default(),
				name: "Instances",
			},
			instance_data.iter().copied(),
		)?;
		let scene = bindless.buffer().alloc_shared_from_data(
			&BindlessBufferCreateInfo {
				usage: BindlessBufferUsage::MAP_WRITE | BindlessBufferUsage::STORAGE_BUFFER,
				allocation_scheme: Default::default(),
				name: "Scene",
			},
			Scene {
				instances: instance_buffer.to_strong(),
				camera,
			},
		)?;

		Ok(CpuScene {
			camera,
			draws,
			instance_total_count,
			scene,
		})
	}
}

pub struct CpuScene {
	pub draws: Vec<CpuDraw>,
	pub instance_total_count: u32,
	pub camera: Camera,
	pub scene: RCDesc<Buffer<Scene>>,
}

pub struct CpuDraw {
	pub model: CpuModel,
	pub instance_start: u32,
	pub instance_count: u32,
}
