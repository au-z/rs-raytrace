// use futures::executor::block_on;

// use winit::{
// 	event::*,
// 	event_loop::{EventLoop, ControlFlow},
// 	window::{WindowBuilder},
// };

// mod ndjinn;
// use crate::ndjinn::Ndjinn;

mod rt;
use rand::prelude::*;
use cgmath::Vector3;
use cgmath::prelude::*;

fn color(ray: &rt::Ray, world: &dyn rt::RTO) -> Vector3<f32> {
	match world.hit(ray, 0.0, f32::MAX) {
		Some(hit) => {
			return 0.5 * Vector3::new(hit.normal.x + 1.0, hit.normal.y + 1.0, hit.normal.z + 1.0);
		},
		_ => {
			let unit_dir = ray.direction.normalize();
			let t = 0.5 * (unit_dir.y + 1.0);
			return (1.0 - t) * Vector3::new(1.0, 1.0, 1.0) + t * Vector3::new(0.5, 0.7, 1.0);
		},
	}
}

fn main() {
	let w = 200;
	let h = 100;
	let samples = 20;
  println!("P3\n{} {}\n255", w, h);

	let camera = rt::camera::Camera::new();

	let objects = vec!(
		Box::new(rt::Sphere::new((0.0, 0.0, -1.0).into(), 0.5)) as Box<dyn rt::RTO>,
		Box::new(rt::Sphere::new((0.0, -100.5, -1.0).into(), 100.0)) as Box<dyn rt::RTO>,
	);
	let world = rt::RTOCollection::new(&objects);

	let mut rng = rand::thread_rng();

	for j in (0..h-1).rev() {
		for i in 0..w {
			let mut col: Vector3<f32> = (0.0, 0.0, 0.0).into(); 
			for _ in 0..samples {
				let rand_u: f32 = rng.gen();
				let rand_v: f32 = rng.gen();
				let u = (i as f32 + rand_u) / w as f32;
				let v = (j as f32 + rand_v) / h as f32;
	
				let ray = camera.get_ray(u, v);
				col += color(&ray, &world);
			}
			col /= samples as f32;
			let r = (256.0 * col.x) as u32;
			let g = (256.0 * col.y) as u32;
			let b = (256.0 * col.z) as u32;
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