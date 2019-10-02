
pub mod prelude {
    pub use super::{Vector3, U8Color, Ray};
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vector3 {
	pub x: f64,
	pub y: f64,
	pub z: f64,
}

impl Vector3 {
	pub fn new(x: f64, y: f64, z: f64) -> Self {
		Vector3 { x, y, z }
	}

	pub fn from_slice<T>(v: [T; 3]) -> Vector3 where T: Into<f64> + Copy {
		Vector3 {
			x: v[0].into(),
			y: v[1].into(),
			z: v[2].into(),
		}
	}

	pub fn normalize(&self) -> Self {
		*self / self.magnitude()
	}

	pub fn magnitude(&self) -> f64 {
		(self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
	}

	pub fn map(&self, f: impl Fn(f64) -> f64) -> Self {
		Vector3 {
			x: f(self.x),
			y: f(self.y),
			z: f(self.z),
		}
	}

	pub fn dot(&self, rhs: Vector3) -> f64 {
		self.x *  rhs.x + self.y * rhs.y + self.z * rhs.z
	}
}

#[repr(C)]
pub struct U8Color {
	r: u8,
	g: u8,
	b: u8,
	a: u8,
}

impl U8Color {
	pub fn as_u32(self) -> u32 {
		self.b as u32 | ((self.g as u32) << 8) | ((self.r as u32) << 16) | ((self.a as u32) << 24)
	}

	pub fn from_vec(from: Vector3, a: u8) -> Self {
		Self {
			r: (from.x * 255.0) as u8,
			g: (from.y * 255.0) as u8,
			b: (from.z * 255.0) as u8,
			a: a
		}
	}
}


use std::ops::{Add, Div, Mul, Sub};


impl Add for Vector3 {
	type Output = Self;

	fn add(self, rhs: Vector3) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
			z: self.z + rhs.z,
		}
	}
}

impl Sub for Vector3 {
	type Output = Self;

	fn sub(self, rhs: Vector3) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
			z: self.z - rhs.z,
		}
	}
}

impl Div<f64> for Vector3 {
	type Output = Self;

	fn div(self, rhs: f64) -> Self::Output {
		self.map(|v| v / rhs)
	}
}

impl Mul<f64> for Vector3 {
	type Output = Self;

	fn mul(self, rhs: f64) -> Self::Output {
		self.map(|v| v * rhs)
	}
}

#[derive(Clone, Copy, Debug)]
pub struct Ray {
	pub origin: Vector3,
	pub direction: Vector3,
}

impl Ray {
	pub fn new(origin: Vector3, direction: Vector3) -> Self {
		Ray { origin, direction }
	}
}

