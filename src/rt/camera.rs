use cgmath::prelude::*;
use cgmath::Vector3;
use rand::prelude::*;
use crate::rt::{Ray};

fn rand_in_unit_disk() -> Vector3<f32> {
  let mut vec: Vector3::<f32>;
  let mut rng = rand::thread_rng();
  loop {
    vec = 2.0 * Vector3::<f32>::new(rng.gen(), rng.gen(), 0.0) - Vector3::<f32>::new(1.0, 1.0, 0.0);
    if vec.dot(vec) < 1.0 { return vec; }
  }
}

pub struct Camera {
  pub origin: Vector3<f32>,
  pub lower_left_corner: Vector3<f32>,
  pub horizontal: Vector3<f32>,
  pub vertical: Vector3<f32>,
  pub lens_r: f32,
  pub u: Vector3<f32>,
  pub v: Vector3<f32>,
  pub w: Vector3<f32>,
}

impl Camera {
  /// Creates a new camera
  pub fn new(origin: Vector3<f32>, target: Vector3<f32>, vfov: f32, aspect: f32, aperture: f32, focus: f32) -> Self {
    let theta = vfov * std::f32::consts::PI / 180.0;
    let half_h = (theta / 2.0).tan();
    let half_w = aspect * half_h;

    let v_up = Vector3::<f32>::unit_y();
    let w = (origin - target).normalize();
    let u = v_up.cross(w).normalize();
    let v = w.cross(u);

    Self {
      origin,
      lower_left_corner: origin - (half_w * focus * u) - (half_h * focus * v) - (focus * w),
      horizontal: 2.0 * half_w * u * focus,
      vertical: 2.0 * half_h * v * focus,
      lens_r: aperture / 2.0,
      u,
      v,
      w,
    }
  }

  pub fn get_ray(&self, s: f32, t: f32) -> Ray {
    let rd = self.lens_r * rand_in_unit_disk();
    let offset = self.u * rd.x + self.v * rd.y;
    return Ray::new(
      self.origin + offset,
      self.lower_left_corner + (s * self.horizontal) + (t * self.vertical) - self.origin - offset,
    );
  }
}