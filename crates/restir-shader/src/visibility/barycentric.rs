use core::ops::{Add, Deref, DerefMut, Mul};
use glam::{Mat2, UVec2, Vec2, Vec3, Vec4, Vec4Swizzles, vec3};
use rust_gpu_bindless_macros::BufferStructPlain;

/// The barycentrics and the derivatives of a Triangle at a certain pixel on the screen
///
/// See http://filmicworlds.com/blog/visibility-buffer-rendering-with-material-graphs/
#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStructPlain)]
pub struct BarycentricDeriv {
	pub lambda: Barycentric,
	pub ddx: Barycentric,
	pub ddy: Barycentric,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, BufferStructPlain)]
pub struct Barycentric(pub Vec3);

impl Deref for Barycentric {
	type Target = Vec3;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for Barycentric {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl BarycentricDeriv {
	pub fn calculate_from(pt0: Vec4, pt1: Vec4, pt2: Vec4, pixel: UVec2, viewport_size: Vec2) -> Self {
		// I added this coordiante system conversion
		let pixel_ndc = pixel.as_vec2() / viewport_size * 2. - 1.;
		let inv_w = Vec3::recip(vec3(pt0.w, pt1.w, pt2.w));

		let ndc0 = pt0.xy() * inv_w.x;
		let ndc1 = pt1.xy() * inv_w.y;
		let ndc2 = pt2.xy() * inv_w.z;

		let inv_det = f32::recip(Mat2::from_cols(ndc2 - ndc1, ndc0 - ndc1).determinant());
		let mut ddx = vec3(ndc1.y - ndc2.y, ndc2.y - ndc0.y, ndc0.y - ndc1.y) * inv_det * inv_w;
		let mut ddy = vec3(ndc2.x - ndc1.x, ndc0.x - ndc2.x, ndc1.x - ndc0.x) * inv_det * inv_w;
		let mut ddx_sum = ddx.element_sum();
		let mut ddy_sum = ddy.element_sum();

		let delta_vec = pixel_ndc - ndc0;
		let interp_inv_w = inv_w.x + delta_vec.x * ddx_sum + delta_vec.y * ddy_sum;
		let interp_w = f32::recip(interp_inv_w);

		let lambda = vec3(
			interp_w * (delta_vec.x * ddx.x + delta_vec.y * ddy.x + inv_w.x),
			interp_w * (delta_vec.x * ddx.y + delta_vec.y * ddy.y),
			interp_w * (delta_vec.x * ddx.z + delta_vec.y * ddy.z),
		);

		ddx *= 2.0 / viewport_size.x;
		ddy *= 2.0 / viewport_size.y;
		ddx_sum *= 2.0 / viewport_size.x;
		ddy_sum *= 2.0 / viewport_size.y;

		ddy *= -1.0;
		ddy_sum *= -1.0;

		let interp_w_ddx = 1.0 / (interp_inv_w + ddx_sum);
		let interp_w_ddy = 1.0 / (interp_inv_w + ddy_sum);

		ddx = interp_w_ddx * (lambda * interp_inv_w + ddx) - lambda;
		ddy = interp_w_ddy * (lambda * interp_inv_w + ddy) - lambda;

		Self {
			lambda: Barycentric(lambda),
			ddx: Barycentric(ddx),
			ddy: Barycentric(ddy),
		}
	}
}

impl Barycentric {
	pub fn interpolate<V: BarycentricInterpolatable>(&self, attr: [V; 3]) -> V {
		V::interpolate(*self, attr)
	}
}

pub trait BarycentricInterpolatable: Sized {
	fn interpolate(bary: Barycentric, attr: [Self; 3]) -> Self;
}

impl<V> BarycentricInterpolatable for V
where
	V: Copy + Mul<f32, Output = V> + Add<V, Output = V>,
{
	fn interpolate(bary: Barycentric, attr: [Self; 3]) -> Self {
		attr[0] * bary.x + attr[1] * bary.y + attr[2] * bary.z
	}
}
