use crate::ndjinn::model;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Light {
  pub position: cgmath::Vector3<f32>,
  pub _padding: u32,
  pub color: cgmath::Vector3<f32>,
}

unsafe impl bytemuck::Zeroable for Light {}
unsafe impl bytemuck::Pod for Light {}

pub trait DrawLight<'a, 'b> where 'b: 'a, {
  fn draw_light_mesh(
    &mut self,
    mesh: &'b model::Mesh,
    uniforms: &'b wgpu::BindGroup,
    light: &'b wgpu::BindGroup,
  );
  fn draw_light_mesh_instanced(
    &mut self,
    mesh: &'b model::Mesh,
    uniforms: &'b wgpu::BindGroup,
    light: &'b wgpu::BindGroup,
    instances: std::ops::Range<u32>,
  ) where 'b: 'a;
  fn draw_light_model(
    &mut self,
    model: &'b model::Model,
    uniforms: &'b wgpu::BindGroup,
    light: &'b wgpu::BindGroup,
  );
  fn draw_light_model_instanced(
    &mut self,
    model: &'b model::Model,
    uniforms: &'b wgpu::BindGroup,
    light: &'b wgpu::BindGroup,
    instances: std::ops::Range<u32>,
  );
}

impl<'a, 'b> DrawLight<'a, 'b> for wgpu::RenderPass<'a> where 'b: 'a, {
  fn draw_light_mesh(
    &mut self,
    mesh: &'b model::Mesh,
    uniforms: &'b wgpu::BindGroup,
    light: &'b wgpu::BindGroup,
  ) {
    self.draw_light_mesh_instanced(mesh, uniforms, light, 0..1);
  }

  fn draw_light_mesh_instanced(
    &mut self,
    mesh: &'b model::Mesh,
    uniforms: &'b wgpu::BindGroup,
    light: &'b wgpu::BindGroup,
    instances: std::ops::Range<u32>,
  ) {
    self.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
    self.set_index_buffer(&mesh.index_buffer, 0, 0);
    self.set_bind_group(0, uniforms, &[]);
    self.set_bind_group(1, light, &[]);
    self.draw_indexed(0..mesh.num_elements, 0, instances);
  }

  fn draw_light_model(
    &mut self,
    model: &'b model::Model,
    uniforms: &'b wgpu::BindGroup,
    light: &'b wgpu::BindGroup,
  ) {
    self.draw_light_model_instanced(model, uniforms, light, 0..1);
  }

  fn draw_light_model_instanced(
    &mut self,
    model: &'b model::Model,
    uniforms: &'b wgpu::BindGroup,
    light: &'b wgpu::BindGroup,
    instances: std::ops::Range<u32>,
  ) {
    for mesh in &model.meshes {
      self.draw_light_mesh_instanced(mesh, uniforms, light, instances.clone());
    }
  }
}
