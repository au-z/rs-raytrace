pub struct Instance {
  pub position: cgmath::Vector3<f32>,
  pub rotation: cgmath::Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct InstanceRaw {
  model: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for InstanceRaw {}
unsafe impl bytemuck::Zeroable for InstanceRaw {}

impl Instance {
  pub fn to_raw(&self) -> InstanceRaw {
    InstanceRaw {
      model: cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation),
    }
  }
}