
use raymarcher_vulkan;
use raymarcher_vulkan::prelude::*;

mod math;
use math::prelude::*;

use minifb::{Key, Window, WindowOptions};

const PI: f64 = ::std::f64::consts::PI;

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




fn main() {
	const WIDTH: usize = 800;
	const HEIGHT: usize = 600;
	const MAX_STEPS: usize = 50;

	let mut window = Window::new(
		"Send help",
		WIDTH,
		HEIGHT,
		WindowOptions::default(),
	)
	.unwrap_or_else(|e| {
		panic!("{}", e);
	});

	let mut time = ::std::time::Instant::now();
	let mut colors =  vec![0; WIDTH*HEIGHT];

	while window.is_open() && !window.is_key_down(Key::Escape) {
		if time.elapsed().as_millis() < 1000 {
			continue;
		}

		time = ::std::time::Instant::now();

		let mut inputs = vec![MarchInstruction::default(); WIDTH*HEIGHT];

		for y in 0..HEIGHT {
			for x in 0..WIDTH {
				let ray = generate_primary_ray((WIDTH, HEIGHT), (x, y), 90.0);
				let origin = [ray.origin.x as f32, ray.origin.y as f32, ray.origin.z as f32];
				let direction = [ray.direction.x as f32, ray.direction.y as f32, ray.direction.z as f32];

				inputs[x + y * WIDTH] = MarchInstruction {
					origin: origin,
					direction: direction,
				}
			}
		}

		let results = raymarcher_vulkan::compute(&inputs.clone());
		
		for y in 0..HEIGHT {
			for x in 0..WIDTH {
				colors[x + y * WIDTH] = (|result: MarchResult, input: MarchInstruction| {
									
					if result.distance < EPSILON as f32 {

							let frag_pos = Vector3::from_slice(input.origin) + Vector3::from_slice(input.direction) * result.distance as f64;
							
							let normal = Vector3::from_slice(result.normal);
							
							let light_pos = Vector3::new(4.0, 3.0, -6.0);
							let light_dir = (light_pos - frag_pos).normalize();
							let light_strength = 10.0;

							let distance = (light_pos - frag_pos).magnitude();
							let attenuation = 1.0 / (distance*distance) * light_strength;

							let cos_theta = light_dir.dot(normal).max(0.0);

							let color = Vector3::new(1.0, 0.0, 0.0) * cos_theta * attenuation + Vector3::new(0.04, 0.04, 0.04);

							return U8Color::from_vec(Vector3::new(1.0, 0.0, 0.0), 255).as_u32()
					}

					0x00000000
				})(results[x + y * WIDTH], inputs[x + y * WIDTH]);
			}
		}

		// We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
		window.update_with_buffer(&colors).unwrap();
		// window.update();
	}

	::std::process::exit(0);

}
