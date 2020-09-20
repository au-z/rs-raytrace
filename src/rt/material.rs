
use cgmath::*;
use rand::prelude::*;
use crate::rt::{Ray, Hit};

fn rand_in_unit_sphere() -> Vector3<f32> {
	let mut vec: Vector3::<f32>;
	let mut rng = rand::thread_rng();
	loop {
		vec = 2.0 * Vector3::<f32>::new(rng.gen(), rng.gen(), rng.gen()) - Vector3::new(1.0, 1.0, 1.0);
		let squared_mag = vec.x*vec.x + vec.y*vec.y + vec.z*vec.z;
		if squared_mag >= 1.0 { return vec; }
	}
}

fn reflect(v: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
  v - 2.0 * v.dot(n) * n
}

// TODO Refraction should vary based on the wavelength of light
fn refract(v: Vector3<f32>, n: Vector3<f32>, index_i: f32, index_r: f32) -> Option<Vector3<f32>> {
  let v_norm = v.normalize();
  let dt = v_norm.dot(n);
  let ratio = index_i / index_r;
  let discriminant = 1.0 - ratio.powf(2.0) * (1.0 - dt.powf(2.0));
  if discriminant > 0.0 {
    return Some(ratio * (v_norm - n * dt) - n * discriminant.sqrt());
  } else {
    return None;
  }
}

/// Estimates Fresnel
fn schlick(cosine: f32, index: f32) -> f32 {
  let r0 = ((1.0 - index) / (1.0 + index)).powf(2.0);
  return r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0);
}

pub trait Material<'a>: MaterialClone<'a> {
  fn scatter(&self, ray: &Ray, hit: &Hit) -> Option<(Ray, Vector3<f32>)>;
}

pub trait MaterialClone<'a> {
  fn clone_box(&self) -> Box<dyn Material<'a>>;
}
impl<'a, T> MaterialClone<'a> for T where T: 'static + Material<'a> + Clone, {
  fn clone_box(&self) -> Box<dyn Material<'a>> {
    Box::new(self.clone())
  }
}

// Lambertian
#[derive(Copy, Clone)]
pub struct Lambertian { pub albedo: Vector3<f32> }
impl<'a> Material<'a> for Lambertian {
  fn scatter(&self, _: &Ray, hit: &Hit) -> Option<(Ray, Vector3<f32>)> {
    let target = hit.position + hit.normal + rand_in_unit_sphere();
    let scattered = Ray::new(hit.position , target - hit.position);
    let attenuation = self.albedo.clone();
    return Some((scattered, attenuation));
  }
}

// Metal
#[derive(Copy, Clone)]
pub struct Metal {
  pub albedo: Vector3<f32>,
  pub roughness: f32,
}
impl Metal {
  #[allow(dead_code)]
  pub fn new(albedo: Vector3<f32>, roughness: f32) -> Self {
    Self {
      albedo,
      roughness: roughness.min(1.0).max(0.0), // 0 <= r >= 1
    }
  }
}

impl<'a> Material<'a> for Metal {
  fn scatter(&self, ray: &Ray, hit: &Hit) -> Option<(Ray, Vector3<f32>)> {
    let reflected = reflect(ray.direction.normalize(), hit.normal);
    let scattered = Ray::new(hit.position, reflected + self.roughness * rand_in_unit_sphere());
    let attenuation = self.albedo;
    if scattered.direction.dot(hit.normal) > 0.0 {
      return Some((scattered, attenuation));
    } else {
      return None;
    }
  }
}

// Dielectric
#[derive(Copy, Clone)]
pub struct Dielectric {
  pub index: f32, // index of refraction
}
impl<'a> Material<'a> for Dielectric {
  fn scatter(&self, ray: &Ray, hit: &Hit) -> Option<(Ray, Vector3<f32>)> {
    let mut outward_normal: Vector3<f32> = hit.normal;
    let mut index_i = 1.0;
    let mut index_r = self.index;
    let mut cosine = -1.0 * ray.direction.dot(hit.normal) / ray.direction.magnitude();

    if ray.direction.dot(hit.normal) > 0.0 {
      outward_normal = -hit.normal;
      index_i = self.index;
      index_r = 1.0;
      cosine = self.index * ray.direction.dot(hit.normal) / ray.direction.magnitude();
    }

    let reflected = reflect(ray.direction.normalize(), hit.normal);
    let attenuation = Vector3::<f32>::new(1.0, 1.0, 1.0); // no glass attenuation

    let mut rng = rand::thread_rng();
    match refract(ray.direction, outward_normal, index_i, index_r) {
      Some(refracted) if rng.gen::<f32>() >= schlick(cosine, self.index) => {
        return Some((Ray::new(hit.position, refracted), attenuation))
      },
      Some(_) | None => {
        return Some((Ray::new(hit.position, reflected), attenuation))
      }
    }
  }
}