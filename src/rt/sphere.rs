use cgmath::{Vector3};
use crate::rt::{Hit, RTO, Ray};
use crate::rt::material::{Material};

pub struct Sphere<'a> {
  center: Vector3<f32>,
  radius: f32,
  material: Box<dyn Material<'a>>,
}

impl<'a> Sphere<'a> {
  pub fn new(center: Vector3<f32>, radius: f32, material: Box<dyn Material<'a>>) -> Self {
    Self {center, radius, material}
  }
}

impl<'a> RTO<'a> for Sphere<'a> {
  fn hit<'b>(&self, ray: &'b Ray, t_min: f32, t_max: f32) -> Option<Hit<'a>> {
    let oc = ray.origin - self.center;
    let a = cgmath::dot(ray.direction, ray.direction);
    let b = cgmath::dot(oc, ray.direction);
    let c = cgmath::dot(oc, oc) - self.radius * self.radius;
    let discriminant = b*b - a*c;
    if discriminant > 0.0 {
      let temp = &[
        (-b - (b*b-a*c).sqrt()) / a,
        (-b + (b*b-a*c).sqrt()) / a,
      ];
      for t in temp {
        if *t < t_max && *t > t_min {
          return Some(Hit {
            distance: *t,
            position: ray.at(*t),
            normal: (ray.at(*t) - self.center) / self.radius,
            material: self.material.clone_box(),
          });
        }
      }
    }
    None
  }
}