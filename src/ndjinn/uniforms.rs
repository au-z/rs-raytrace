use cgmath::prelude::*;
use crate::ndjinn::camera::{Camera, Projection};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Uniforms {
	view_position: cgmath::Vector4<f32>,
	view_proj: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for Uniforms {}
unsafe impl bytemuck::Zeroable for Uniforms {}

impl Uniforms {
	pub fn new() -> Self {
		Self {
			view_position: Zero::zero(),
			view_proj: cgmath::Matrix4::identity(),
		}
	}

	pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
		self.view_position = camera.position.to_homogeneous();
		self.view_proj = projection.calc_matrix() * camera.calc_matrix();
	}
}

