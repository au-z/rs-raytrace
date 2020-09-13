use cgmath::Vector3;
use crate::rt;

pub struct Camera {
  pub lower_left_corner: Vector3<f32>,
  pub horizontal: Vector3<f32>,
  pub vertical: Vector3<f32>,
  pub origin: Vector3<f32>,
}

impl Camera {
  pub fn new() -> Self {
    Self {
      lower_left_corner: (-2.0, -1.0, -1.0).into(),
      horizontal: (4.0, 0.0, 0.0).into(),
      vertical: (0.0, 2.0, 0.0).into(),
      origin: (0.0, 0.0, 0.0).into(),
    }
  }

  pub fn get_ray(&self, u: f32, v: f32) -> rt::Ray {
    return rt::Ray::new(self.origin, self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin);
  }
}