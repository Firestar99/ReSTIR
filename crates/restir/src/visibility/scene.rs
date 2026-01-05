use crate::model::VisiCpuModel;
use restir_shader::camera::Camera;
use restir_shader::visibility::id::InstanceId;
use restir_shader::visibility::scene::{VisiInstance, VisiInstanceInfo, VisiScene};
use rust_gpu_bindless::descriptor::{
	Bindless, BindlessBufferCreateInfo, BindlessBufferUsage, Buffer, RCDesc, RCDescExt,
};
use std::collections::HashMap;

pub struct VisiCpuSceneAccum {
	pub instances: HashMap<VisiCpuModel, Vec<VisiInstance>>,
}

impl Default for VisiCpuSceneAccum {
	fn default() -> Self {
		Self::new()
	}
}

impl VisiCpuSceneAccum {
	pub fn new() -> Self {
		Self {
			instances: HashMap::new(),
		}
	}

	pub fn push(&mut self, model: &VisiCpuModel, instance: VisiInstanceInfo) {
		let instance = VisiInstance {
			model: model.model.to_strong(),
			info: instance,
		};
		self.instances.entry(model.clone()).or_default().push(instance);
	}

	pub fn finish(self, bindless: &Bindless, camera: Camera) -> anyhow::Result<VisiCpuScene> {
		let mut instance_data = Vec::with_capacity(self.instances.values().map(|i| i.len()).sum());
		let draws = self
			.instances
			.into_iter()
			.map(|(model, instances)| {
				let instance_start = instance_data.len() as u32;
				let instance_count = instances.len() as u32;
				// verify no oob in shaders later
				InstanceId::new(instance_start + instance_count)?;
				instance_data.extend(instances.into_iter());
				Ok(VisiCpuDraw {
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
			VisiScene {
				instances: instance_buffer.to_strong(),
				camera,
			},
		)?;

		Ok(VisiCpuScene {
			camera,
			draws,
			instance_total_count,
			scene,
		})
	}
}

pub struct VisiCpuScene {
	pub draws: Vec<VisiCpuDraw>,
	pub instance_total_count: u32,
	pub camera: Camera,
	pub scene: RCDesc<Buffer<VisiScene>>,
}

pub struct VisiCpuDraw {
	pub model: VisiCpuModel,
	pub instance_start: u32,
	pub instance_count: u32,
}
