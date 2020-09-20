// use futures::executor::block_on;

// use winit::{
// 	event::*,
// 	event_loop::{EventLoop, ControlFlow},
// 	window::{WindowBuilder},
// };

// mod ndjinn;
// use crate::ndjinn::Ndjinn;

use cgmath::Vector3;
use cgmath::prelude::*;
use rand::prelude::*;
mod rt;
use rt::{RTOCollection, RTO, ray::Ray, sphere::Sphere, camera::Camera};
use rt::material::{Lambertian, Metal, Dielectric};

const WIDTH: u32 = 80;
const MAX_BOUNCES: u32 = 12;
const AA_SAMPLES: u32 = 50;
const HEIGHT: u32 = 80;

fn color<'a, 'b>(ray: &'b Ray, world: &dyn RTO<'a>, depth: u32) -> Vector3<f32> {
	match world.hit(&ray, 0.001, f32::MAX) {
		Some(hit) => {
			if depth < MAX_BOUNCES {
				match hit.material.scatter(&ray, &hit) {
					Some((scattered_ray, attenuation)) => {
						let scattered_color = color(&scattered_ray, world, depth+1);
						return (
							attenuation.x * scattered_color.x,
							attenuation.y * scattered_color.y,
							attenuation.z * scattered_color.z,
						).into();
					}
					_ => {
						return (0.0, 0.0, 0.0).into();
					}
				}
			} else {
				return (0.0, 0.0, 0.0).into();
			}
		},
		_ => {
			let unit_dir = ray.direction.normalize();
			let t = 0.5 * (unit_dir.y + 1.0);
			return (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0);
		},
	}
}

fn main() {
	let mut rng = rand::thread_rng();
	let w = WIDTH;
	let h = HEIGHT;
	let samples = AA_SAMPLES;
  println!("P3\n{} {}\n255", w, h);

	let origin = Vector3::<f32>::new(11.0, 2.0, 3.0);
	let target = Vector3::<f32>::new(0.0, 0.0, -1.0);
	let aspect = w as f32 / h as f32;
	let focus = (origin - target).magnitude();
	let camera = Camera::new(origin, target, 45.0, aspect, 0.0, focus);

	let mut objects = vec!(
		Box::new(Sphere::new((0.0, -1000.0, -1.0).into(), 1000.0, Box::new(Lambertian {
			albedo: (0.5, 0.5, 0.5).into(),
		}))) as Box<dyn RTO>,
	);

	for a in -11..11 {
		for b in -11..11 {
			let mat_code: f32 = rng.gen();
			let center = Vector3::<f32>::new(a as f32 + 0.9 * rng.gen::<f32>(), 0.2, b as f32 + 0.9 * rng.gen::<f32>());
			if (center - Vector3::<f32>::new(4.0, 0.2, 0.0)).magnitude() > 0.9 {
				if mat_code < 0.7 { // diffuse
					objects.push(Box::new(Sphere::new(center, 0.2, Box::new(Lambertian {
						albedo: Vector3::<f32>::new(rng.gen::<f32>() * rng.gen::<f32>(), rng.gen::<f32>() * rng.gen::<f32>(), rng.gen::<f32>() * rng.gen::<f32>()),
					}))) as Box<dyn RTO>);
				} else if mat_code < 0.9 { // metal
					objects.push(Box::new(Sphere::new(center, 0.2, Box::new(Metal {
						albedo: Vector3::<f32>::new(0.5 * (1.0 + rng.gen::<f32>()), 0.5 * (1.0 + rng.gen::<f32>()), 0.5 * (1.0 + rng.gen::<f32>())),
						roughness: 0.5 * (1.0 + rng.gen::<f32>()),
					}))) as Box<dyn RTO>);
				} else { // glass
					objects.push(Box::new(Sphere::new(center, 0.2, Box::new(Dielectric {
						index: 1.5,
					}))) as Box<dyn RTO>);
				}
			}
		}
	}

	objects.push(Box::new(Sphere::new(Vector3::<f32>::new(0.0, 1.0, 0.0), 1.0, Box::new(Dielectric {
		index: 1.5,
	}))) as Box<dyn RTO>);
	objects.push(Box::new(Sphere::new(Vector3::<f32>::new(-4.0, 1.0, 0.0), 1.0, Box::new(Lambertian {
		albedo: (0.4, 0.2, 0.1).into(),
	}))) as Box<dyn RTO>);
	objects.push(Box::new(Sphere::new(Vector3::<f32>::new(4.0, 1.0, 0.0), 1.0, Box::new(Metal {
		albedo: (0.7, 0.6, 0.5).into(),
		roughness: 0.0,
	}))) as Box<dyn RTO>);

	let world = RTOCollection::new(objects);

	// let world = RTOCollection::new(vec!(
	// 	// FLOOR
	// 	Box::new(Sphere::new((0.0, -100.5, -1.0).into(), 100.0, Box::new(Lambertian {
	// 		albedo: (0.04, 0.06, 0.09).into(),
	// 	}))) as Box<dyn RTO>,
	// 	// LEFT
	// 	Box::new(Sphere::new((-1.0, 0.0, -1.0).into(), 0.5, Box::new(Dielectric {
	// 		index: 1.5,
	// 	}))) as Box<dyn RTO>,
	// 	Box::new(Sphere::new((-1.0, 0.0, -1.0).into(), -0.48, Box::new(Dielectric {
	// 		index: 1.5,
	// 	}))) as Box<dyn RTO>,
	// 	// CENTER
	// 	Box::new(Sphere::new((0.0, 0.0, -1.0).into(), 0.5, Box::new(Lambertian {
	// 		albedo: (0.1, 0.2, 0.5).into(),
	// 	}))) as Box<dyn RTO>,
	// 	// RIGHT
	// 	Box::new(Sphere::new((1.0, 0.0, -1.0).into(), 0.5, Box::new(Metal::new(
	// 		(0.8, 0.6, 0.3).into(),
	// 		0.0,
	// 	)))) as Box<dyn RTO>,
	// ));

	for j in (0..h-1).rev() {
		for i in 0..w {
			let mut col: Vector3<f32> = (0.0, 0.0, 0.0).into(); 
			for _ in 0..samples {
				let rand_u: f32 = rng.gen();
				let rand_v: f32 = rng.gen();
				let u = (i as f32 + rand_u) / w as f32;
				let v = (j as f32 + rand_v) / h as f32;
	
				let ray = camera.get_ray(u, v);
				col += color(&ray, &world, 0);
			}
			col /= samples as f32;
			// gamma correction
			col = (col.x.powf(0.5), col.y.powf(0.5), col.z.powf(0.5)).into();

			let r = (255.99 * col.x) as u32;
			let g = (255.99 * col.y) as u32;
			let b = (255.99 * col.z) as u32;
			println!("{} {} {}", r, g, b);
		}
	}
}
// 	let event_loop = EventLoop::new();
// 	let window = WindowBuilder::new().build(&event_loop).unwrap();

// 	let mut ndjinn = block_on(Ndjinn::new(&window));
// 	let mut last_render_time = std::time::Instant::now();

// 	event_loop.run(move |event, _, control_flow| {
// 		*control_flow = ControlFlow::Poll;
// 		match event {
// 			Event::WindowEvent {ref event, window_id} if window_id == window.id() => if !ndjinn.input(event) {
// 				match event {
// 					WindowEvent::Resized(physical_size) => {
// 						ndjinn.resize(*physical_size);
// 					}
// 					WindowEvent::ScaleFactorChanged {new_inner_size, ..} => {
// 						ndjinn.resize(**new_inner_size);
// 					}
// 					WindowEvent::KeyboardInput {input, ..} => {
// 						match input {
// 							KeyboardInput {
// 								state: ElementState::Pressed,
// 								virtual_keycode: Some(VirtualKeyCode::Escape),
// 								..
// 							} => *control_flow = ControlFlow::Exit,
// 							_ => {}
// 						}
// 					}
// 					WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
// 					_ => {}
// 				}
// 			}
// 			Event::RedrawRequested(_) => {
// 				let now = std::time::Instant::now();
// 				let dt = now - last_render_time;
// 				last_render_time = now;
// 				ndjinn.update(dt);
// 				ndjinn.render();
// 			}
// 			Event::MainEventsCleared => {
// 				window.request_redraw();
// 			}
// 			_ => {}
// 		}
// 	});
// }