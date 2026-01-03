use core::ops::{Div, Sub};
use rust_gpu_bindless_macros::BufferStruct;
use spirv_std::num_traits::{Euclid, One};

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStruct)]
pub struct DebugValueRange {
	pub min: i32,
	pub max: i32,
	pub wrap: bool,
}

impl Default for DebugValueRange {
	fn default() -> Self {
		Self {
			min: 0,
			max: 32,
			wrap: true,
		}
	}
}

impl DebugValueRange {
	pub fn clamp<V: Sub<f32, Output = V> + Div<f32, Output = V> + Euclid + One>(&self, value: V) -> V {
		let mut out = (value - self.min as f32) / (self.max - self.min) as f32;
		if self.wrap {
			out = V::rem_euclid(&out, &V::one());
		}
		out
	}
}
