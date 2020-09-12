use futures::executor::block_on;

use winit::{
	event::*,
	event_loop::{EventLoop, ControlFlow},
	window::{WindowBuilder},
};

mod ndjinn;
use crate::ndjinn::Ndjinn;

fn main() {
	let event_loop = EventLoop::new();
	let window = WindowBuilder::new().build(&event_loop).unwrap();

	let mut ndjinn = block_on(Ndjinn::new(&window));
	let mut last_render_time = std::time::Instant::now();

	event_loop.run(move |event, _, control_flow| {
		*control_flow = ControlFlow::Poll;
		match event {
			Event::WindowEvent {ref event, window_id} if window_id == window.id() => if !ndjinn.input(event) {
				match event {
					WindowEvent::Resized(physical_size) => {
						ndjinn.resize(*physical_size);
					}
					WindowEvent::ScaleFactorChanged {new_inner_size, ..} => {
						ndjinn.resize(**new_inner_size);
					}
					WindowEvent::KeyboardInput {input, ..} => {
						match input {
							KeyboardInput {
								state: ElementState::Pressed,
								virtual_keycode: Some(VirtualKeyCode::Escape),
								..
							} => *control_flow = ControlFlow::Exit,
							_ => {}
						}
					}
					WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
					_ => {}
				}
			}
			Event::RedrawRequested(_) => {
				let now = std::time::Instant::now();
				let dt = now - last_render_time;
				last_render_time = now;
				ndjinn.update(dt);
				ndjinn.render();
			}
			Event::MainEventsCleared => {
				window.request_redraw();
			}
			_ => {}
		}
	});
}
