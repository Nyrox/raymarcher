const PI: f64 = std::f64::consts::PI;
const EPSILON: f64 = 0.001;


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vector3 {
	x: f64,
	y: f64,
	z: f64,
}

impl Vector3 {
	pub fn new(x: f64, y: f64, z: f64) -> Self {
		Vector3 { x, y, z }
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
struct U8Color {
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
	origin: Vector3,
	direction: Vector3,
}

impl Ray {
	pub fn new(origin: Vector3, direction: Vector3) -> Self {
		Ray { origin, direction }
	}
}

fn generate_primary_ray((width, height): (usize, usize), (x, y): (usize, usize), fov: f64) -> Ray {
	let width = width as f64;
	let height = height as f64;
	let aspect = width / height;
	let x = x as f64;
	let y = y as f64;

	let px = (2.0 * ((x + 0.5) / width) - 1.0) * f64::tan(fov / 2.0 * PI / 180.0) * aspect;
	let py = (1.0 - 2.0 * ((y + 0.5) / height)) * f64::tan(fov / 2.0 * PI / 180.0);

	Ray::new(
		Vector3::new(0.0, 0.0, -10.0),
		Vector3::new(px, py, 1.0).normalize(),
	)
}

pub mod sdf {
	use super::*;


	pub fn sphere(radius: f64) -> impl Fn(Vector3) -> f64 {
		move |p| {
			p.magnitude() - radius
		}
	}

	pub fn translate(sdf: impl Fn(Vector3) -> f64, translation: Vector3) -> impl Fn(Vector3) -> f64 {
		move |p| {
			sdf(p - translation)
		}
	}

	pub fn max(s1: impl Fn(Vector3) -> f64, s2: impl Fn(Vector3) -> f64) -> impl Fn(Vector3) -> f64 {
		move |p| {
			s1(p).max(s2(p))
		}
	}

	pub fn min(s1: impl Fn(Vector3) -> f64, s2: impl Fn(Vector3) -> f64) -> impl Fn(Vector3) -> f64 {
		move |p| {
			s1(p).min(s2(p))
		}
	}

	fn mix(a: f64, b: f64, m: f64) -> f64 {
		a + ((b - a) * m)
	}

	pub fn smooth_min(s1: impl Fn(Vector3) -> f64, s2: impl Fn(Vector3) -> f64, k: f64) -> impl Fn(Vector3) -> f64 {
		move |p| {
			let (a, b) = (s1(p), s2(p));
			let h = (0.5+0.5*(b-a)/k).min(1.0).max(0.0);
			return mix(b, a, h) - k*h*(1.0-h);
		}
	}

	pub fn difference(s1: impl Fn(Vector3) -> f64, s2: impl Fn(Vector3) -> f64) -> impl Fn(Vector3) -> f64 {
		move |p| {
			s1(p).max(-s2(p))
		}
	}
}

use minifb::{Key, Window, WindowOptions};

fn scene(pos: Vector3) -> f64 {
	let sdf = {
		sdf::difference(
		sdf::smooth_min(
			sdf::sphere(3.0),
			sdf::translate(
				sdf::sphere(2.0),
				Vector3::new(0.0, 3.5, 0.0),
			),
			1.0
		),
		sdf::translate(
			sdf::sphere(2.5),
			Vector3::new(1.5, 1.5, -1.75),
		))
	};

	sdf(pos)
}


fn estimate_normal(pos: Vector3) -> Vector3 {
	Vector3::new(
		scene(pos + Vector3::new(EPSILON, 0.0, 0.0)) - scene(pos - Vector3::new(EPSILON, 0.0, 0.0)),
		scene(pos + Vector3::new(0.0, EPSILON, 0.0)) - scene(pos - Vector3::new(0.0, EPSILON, 0.0)),
		scene(pos + Vector3::new(0.0, 0.0, EPSILON)) - scene(pos - Vector3::new(0.0, 0.0, EPSILON))
	).normalize()
}

fn main() {
	const WIDTH: usize = 600;
	const HEIGHT: usize = 600;
	const MAX_STEPS: usize = 50;

	let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

	let mut window = Window::new(
		"Test - ESC to exit",
		WIDTH,
		HEIGHT,
		WindowOptions::default(),
	)
	.unwrap_or_else(|e| {
		panic!("{}", e);
	});

	let mut time = ::std::time::Instant::now();

	while window.is_open() && !window.is_key_down(Key::Escape) {
		if time.elapsed().as_millis() < 200 {
			continue;
		}
		time = ::std::time::Instant::now();

		for y in 0..HEIGHT {
			for x in 0..WIDTH {
				buffer[x + y * WIDTH] = (|| {
					let mut depth = EPSILON;
					let ray = generate_primary_ray((WIDTH, HEIGHT), (x, y), 64.0);

					for _ in 0..MAX_STEPS {
						let frag_pos = ray.origin + ray.direction * depth;
						let dist = scene(frag_pos);

						if dist < EPSILON {
							// were inside the surface
							let normal = estimate_normal(frag_pos);

							let light_pos = Vector3::new(4.0, 3.0, -6.0);
							let light_dir = (light_pos - frag_pos).normalize();
							let light_strength = 10.0;

							let distance = (light_pos - frag_pos).magnitude();
							let attenuation = 1.0 / (distance*distance) * light_strength;

							let cos_theta = light_dir.dot(normal).max(0.0);

							let color = Vector3::new(1.0, 0.0, 0.0) * cos_theta * attenuation + Vector3::new(0.04, 0.04, 0.04);
							return U8Color::from_vec(color, 255).as_u32()
						}

						depth += dist;
					}

					0x000000
				})()
			}
		}


		// We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
		window.update_with_buffer(&buffer).unwrap();
	}

}
