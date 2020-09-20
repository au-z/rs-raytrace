use cgmath::Vector3;
pub mod ppm;
pub mod camera;
pub mod material;
use material::{Material};
pub mod ray;
use ray::{Ray};
pub mod sphere;

pub struct Hit<'a> {
  pub distance: f32,
  pub position: Vector3<f32>,
  pub normal: Vector3<f32>,
  pub material: Box<dyn Material<'a>>,
}

/// Ray Traced Object
pub trait RTO<'a> {
  fn hit<'b>(&self, ray: &'b Ray, t_min: f32, t_max: f32) -> Option<Hit<'a>>;
}

pub struct RTOCollection<'a> {
  collection: Vec<Box<dyn RTO<'a>>>,
}

impl<'a> RTOCollection<'a> {
  pub fn new(collection: Vec<Box<dyn RTO<'a>>>) -> Self {
    Self {collection}
  }
}

impl<'a> RTO<'a> for RTOCollection<'a> {
  fn hit<'b>(&self, ray: &'b Ray, t_min: f32, t_max: f32) -> Option<Hit<'a>> {
    let mut closest_hit = t_max;
    let mut result: Option<Hit<'a>> = None;
    for obj in &self.collection {
      match obj.hit(ray, t_min, closest_hit) {
        Some(hit) => {
          closest_hit = hit.distance;
          result = Some(hit);
        },
        _ => {},
      }
    }
    result
  }
}