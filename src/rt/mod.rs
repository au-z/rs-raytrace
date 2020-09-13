pub mod ppm;
pub mod camera;
use cgmath::Vector3;

pub struct Ray {
  pub origin: Vector3<f32>,
  pub direction: Vector3<f32>,
}

impl Ray {
  pub fn new(origin: Vector3<f32>, direction: Vector3<f32>) -> Self {
    Self {
      origin,
      direction,
    }
  }

  pub fn at(&self, t: f32) -> Vector3<f32> {
    self.origin + t * self.direction
  }
}

pub struct Hit {
  pub distance: f32,
  pub position: Vector3<f32>,
  pub normal: Vector3<f32>,
}

/// Ray Traced Object
pub trait RTO {
  fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit>;
}

pub struct Sphere {
  center: Vector3<f32>,
  radius: f32,
}

impl Sphere {
  pub fn new(center: Vector3<f32>, radius: f32) -> Self {
    Self {center, radius}
  }
}

impl RTO for Sphere {
  fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
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
          });
        }
      }
    }
    None
  }
}

pub struct RTOCollection<'a> {
  collection: &'a Vec<Box<dyn RTO>>,
}

impl<'a> RTOCollection<'a> {
  pub fn new(collection: &'a Vec<Box<dyn RTO>>) -> Self {
    Self {collection}
  }
}

impl RTO for RTOCollection<'_> {
  fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<Hit> {
    let mut closest_hit = t_max;
    let mut result: Option<Hit> = None;
    for obj in self.collection {
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