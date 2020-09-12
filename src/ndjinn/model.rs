use std::path::Path;
use crate::ndjinn::texture;
use crate::ndjinn::gpu;

pub trait Vertex {
	fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ModelVertex {
	position: cgmath::Vector3<f32>,
	uv: cgmath::Vector2<f32>,
	normal: cgmath::Vector3<f32>,
	tangent: cgmath::Vector3<f32>,
	bitangent: cgmath::Vector3<f32>,
}

unsafe impl bytemuck::Pod for ModelVertex {}
unsafe impl bytemuck::Zeroable for ModelVertex {}

impl Vertex for ModelVertex {
	fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
		wgpu::VertexBufferDescriptor {
			stride: std::mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
			step_mode: wgpu::InputStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttributeDescriptor {
					offset: 0,
					shader_location: 0,
					format: wgpu::VertexFormat::Float3,
				},
				wgpu::VertexAttributeDescriptor {
					offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
					shader_location: 1,
					format: wgpu::VertexFormat::Float2,
				},
				wgpu::VertexAttributeDescriptor {
					offset: std::mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
					shader_location: 2,
					format: wgpu::VertexFormat::Float3,
				},
				wgpu::VertexAttributeDescriptor {
					offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
					shader_location: 3,
					format: wgpu::VertexFormat::Float3,
				},
				wgpu::VertexAttributeDescriptor {
					offset: std::mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
					shader_location: 4,
					format: wgpu::VertexFormat::Float3,
				},
			],
		}
	}
}

pub struct Material {
	pub name: String,
	pub diffuse_texture: texture::Texture,
	pub normal_texture: texture::Texture,
	pub bind_group: gpu::BindGroup,
}

impl Material {
	pub fn new(
		device: &wgpu::Device,
		name: &str,
		diffuse_texture: texture::Texture,
		normal_texture: texture::Texture,
	) -> Self {
		let bind_group = gpu::BindGroup::create(&device, &[
			// DIFFUSE TEXTURE
			gpu::BindingConfiguration {
				datatype: gpu::ResourceType::Texture {view: &diffuse_texture.view},
				stage: wgpu::ShaderStage::FRAGMENT,
				binding_type: wgpu::BindingType::SampledTexture {
					multisampled: false,
					component_type: wgpu::TextureComponentType::Float,
					dimension: wgpu::TextureViewDimension::D2,
				},
			},
			gpu::BindingConfiguration {
				datatype: gpu::ResourceType::Sampler {sampler: &diffuse_texture.sampler},
				stage: wgpu::ShaderStage::FRAGMENT,
				binding_type: wgpu::BindingType::Sampler {comparison: false},
			},
			// NORMAL TEXTURE
			gpu::BindingConfiguration {
				datatype: gpu::ResourceType::Texture {view: &normal_texture.view},
				stage: wgpu::ShaderStage::FRAGMENT,
				binding_type: wgpu::BindingType::SampledTexture {
					multisampled: false,
					component_type: wgpu::TextureComponentType::Float,
					dimension: wgpu::TextureViewDimension::D2,
				},
			},
			gpu::BindingConfiguration {
				datatype: gpu::ResourceType::Sampler {sampler: &normal_texture.sampler},
				stage: wgpu::ShaderStage::FRAGMENT,
				binding_type: wgpu::BindingType::Sampler {comparison: false},
			}
		], Some(name));

		Self {
			name: String::from(name),
			diffuse_texture,
			normal_texture,
			bind_group,
		}
	}
}

pub struct Mesh {
	pub name: String,
	pub vertex_buffer: wgpu::Buffer,
	pub index_buffer: wgpu::Buffer,
	pub num_elements: u32,
	pub material: usize,
}

pub struct Model {
	pub meshes: Vec<Mesh>,
	pub materials: Vec<Material>,
}

impl Model {
	pub fn load<P: AsRef<Path>>(device: &wgpu::Device, path: P) -> Result<(Self, Vec<wgpu::CommandBuffer>), failure::Error> {
		let (obj_models, obj_materials) = tobj::load_obj(path.as_ref()).expect("Failed to load file.");

		let parent_path = path.as_ref().parent().unwrap();

		let mut command_buffers = Vec::new();

		let mut materials = Vec::new();
		for mat in obj_materials {
			let diffuse_path = parent_path.join(mat.diffuse_texture);
			let (diffuse_texture, cmds) = texture::Texture::load(&device, diffuse_path, false)
				.expect("Failed to load diffuse texture.");
			command_buffers.push(cmds);
			
			let normal_path = match mat.unknown_param.get("map_Bump") {
				Some(v) => Ok(v),
				None => Err(failure::err_msg("Unable to find normal map"))
			};
			let (normal_texture, cmds) = texture::Texture::load(&device, parent_path.join(normal_path?), true)
				.expect("Failed to load normal texture.");
			command_buffers.push(cmds);

			materials.push(Material::new(
				&device,
				&mat.name,
				diffuse_texture,
				normal_texture,
			));
		}

		let mut meshes = Vec::new();
		for m in obj_models {
			let mut vertices = Vec::new();
			for i in 0..m.mesh.positions.len() / 3 {
				vertices.push(ModelVertex {
					position: [m.mesh.positions[i * 3], m.mesh.positions[i * 3 + 1], m.mesh.positions[i * 3 + 2]].into(),
					uv: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]].into(),
					normal: [m.mesh.normals[i * 3], m.mesh.normals[i * 3 + 1], m.mesh.normals[i * 3 + 2]].into(),
					tangent: [0.0; 3].into(),
					bitangent: [0.0; 3].into(),
				});
			}

			// calculate tangent and bitangent vectors
			let indices = &m.mesh.indices;
			for c in indices.chunks(3) {
				let v0 = vertices[c[0] as usize];
				let v1 = vertices[c[1] as usize];
				let v2 = vertices[c[2] as usize];

				let pos0 = v0.position;
				let pos1 = v1.position;
				let pos2 = v2.position;

				let uv0 = v0.uv;
				let uv1 = v1.uv;
				let uv2 = v2.uv;

				// edges of the triangle
				let delta_pos1 = pos1 - pos0;
				let delta_pos2 = pos2 - pos0;

				// direction for the tangent/bitangent
				let delta_uv1 = uv1 - uv0;
				let delta_uv2 = uv2 - uv0;

				// Solving the following system of equations will
				// give us the tangent and bitangent.
				//   delta_pos1 = delta_uv1.x * T + delta_u.y * B
				//   delta_pos2 = delta_uv2.x * T + delta_uv2.y * B
				// Luckily, the place I found this equation provided 
				// the solution!
				let r = 1.0 / (delta_uv1 .x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
				let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
				let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;
				
				// We'll use the same tangent/bitangent for each vertex in the triangle
				vertices[c[0] as usize].tangent = tangent;
				vertices[c[1] as usize].tangent = tangent;
				vertices[c[2] as usize].tangent = tangent;

				vertices[c[0] as usize].bitangent = bitangent;
				vertices[c[1] as usize].bitangent = bitangent;
				vertices[c[2] as usize].bitangent = bitangent;
			}

			let vertex_buffer = device.create_buffer_with_data(
				bytemuck::cast_slice(&vertices),
				wgpu::BufferUsage::VERTEX,
			);
			let index_buffer = device.create_buffer_with_data(
				bytemuck::cast_slice(&m.mesh.indices), 
				wgpu::BufferUsage::INDEX,
			);

			meshes.push(Mesh {
				name: m.name,
				vertex_buffer,
				index_buffer,
				num_elements: m.mesh.indices.len() as u32,
				material: m.mesh.material_id.unwrap_or(0),
			});
		}

		Ok((Self {meshes, materials}, command_buffers))
	}
}

pub trait DrawModel<'a, 'b> where 'b: 'a, {
	fn draw_mesh(
		&mut self,
		mesh: &'b Mesh,
		material: &'b Material,
		uniforms: &'b wgpu::BindGroup,
		light: &'b wgpu::BindGroup,
	);
	fn draw_mesh_instanced(
		&mut self,
		mesh: &'b Mesh,
		material: &'b Material,
		uniforms: &'b wgpu::BindGroup,
		light: &'b wgpu::BindGroup,
		instances: std::ops::Range<u32>,
	);
	fn draw_model(
		&mut self,
		model: &'b Model,
		uniforms: &'b wgpu::BindGroup,
		light: &'b wgpu::BindGroup,
	);
	fn draw_model_instanced(
		&mut self,
		model: &'b Model,
		uniforms: &'b wgpu::BindGroup,
		light: &'b wgpu::BindGroup,
		instances: std::ops::Range<u32>,
	);
	fn draw_model_instanced_with_material(
		&mut self,
		model: &'b Model,
		material: &'b Material,
		uniforms: &'b wgpu::BindGroup,
		light: &'b wgpu::BindGroup,
		instances: std::ops::Range<u32>,
	);
}

impl<'a, 'b> DrawModel<'a, 'b> for wgpu::RenderPass<'a> where 'b: 'a, {
	fn draw_mesh(
		&mut self,
		mesh: &'b Mesh,
		material: &'b Material,
		uniforms: &'b wgpu::BindGroup,
		light: &'b wgpu::BindGroup,
	) {
		self.draw_mesh_instanced(mesh, material, uniforms, light, 0..1);
	}

	fn draw_mesh_instanced(
		&mut self,
		mesh: &'b Mesh,
		material: &'b Material,
		uniforms: &'b wgpu::BindGroup,
		light: &'b wgpu::BindGroup,
		instances: std::ops::Range<u32>,
	) {
		self.set_vertex_buffer(0, &mesh.vertex_buffer, 0, 0);
		self.set_index_buffer(&mesh.index_buffer, 0, 0);
		self.set_bind_group(0, &material.bind_group.data, &[]);
		self.set_bind_group(1, &uniforms, &[]);
		self.set_bind_group(2, &light, &[]);
		self.draw_indexed(0..mesh.num_elements, 0, instances);
	}

	fn draw_model(
		&mut self,
		model: &'b Model,
		uniforms: &'b wgpu::BindGroup,
		light: &'b wgpu::BindGroup,
	) {
		self.draw_model_instanced(model, uniforms, light, 0..1);
	}

	fn draw_model_instanced(
		&mut self,
		model: &'b Model,
		uniforms: &'b wgpu::BindGroup,
		light: &'b wgpu::BindGroup,
		instances: std::ops::Range<u32>,
	) {
		for mesh in &model.meshes {
			let material = &model.materials[mesh.material];
			self.draw_mesh_instanced(mesh, material, uniforms, light, instances.clone());
		}
	}

	fn draw_model_instanced_with_material(
		&mut self,
		model: &'b Model,
		material: &'b Material,
		uniforms: &'b wgpu::BindGroup,
		light: &'b wgpu::BindGroup,
		instances: std::ops::Range<u32>,
	) {
		for mesh in &model.meshes {
			self.draw_mesh_instanced(mesh, material, uniforms, light, instances.clone());
		}
	}
}