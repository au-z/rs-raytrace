use cgmath::prelude::*;

use winit::{
	event::*,
	window::{Window},
	dpi::PhysicalPosition,
};

mod camera;
mod gpu;
mod instance;
mod light;
use light::{DrawLight};
mod model;
use model::{DrawModel, Vertex};
mod shader;
mod texture;
mod uniforms;
use uniforms::{Uniforms};

pub struct Ndjinn {
  surface: wgpu::Surface,
  device: wgpu::Device,
  queue: wgpu::Queue,
  sc_desc: wgpu::SwapChainDescriptor,
  swap_chain: wgpu::SwapChain,
  depth_texture: texture::Texture,
  light: light::Light,
  light_buffer: wgpu::Buffer,
  light_bind_group: wgpu::BindGroup,
  camera: camera::Camera,
  projection: camera::Projection,
  camera_controller: camera::CameraController,
  last_mouse_pos: PhysicalPosition<f64>,
  mouse_pressed: bool,
  obj_model: model::Model,
  render_pipeline: wgpu::RenderPipeline,
  light_render_pipeline: wgpu::RenderPipeline,
  uniforms: Uniforms,
  uniform_buffer: wgpu::Buffer,
  uniform_bind_group: wgpu::BindGroup,
  instances: Vec<instance::Instance>,
  size: winit::dpi::PhysicalSize<u32>,
}

impl Ndjinn {
  pub async fn new(window: &Window) -> Self {
    const NUM_INSTANCES_PER_ROW: u32 = 1;
    const SPACE_BETWEEN: f32 = 3.0;
    
    // Setup
    let size = window.inner_size();
    let surface = wgpu::Surface::create(window);

    let adapter = wgpu::Adapter::request(
      &wgpu ::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::Default,
        compatible_surface: Some(&surface),
      },
      wgpu::BackendBit::PRIMARY, // Vulkan + Metal + DX12 + Browser WebGPU
    ).await.unwrap();

    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
      extensions: wgpu::Extensions {anisotropic_filtering: false},
      limits: Default::default(),
    }).await;

    let sc_desc = wgpu::SwapChainDescriptor {
      usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
      format: wgpu::TextureFormat::Bgra8UnormSrgb,
      width: size.width,
      height: size.height,
      present_mode: wgpu::PresentMode::Fifo,
    };
    let swap_chain = device.create_swap_chain(&surface, &sc_desc);

    // Depth Buffer
    let depth_texture = texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

    // Light
    let light = light::Light {
      position: (2.0, 2.0, 2.0).into(),
      _padding: 0,
      color: (1.0, 1.0, 1.0).into(),
    };

    let light_buffer = device.create_buffer_with_data(
      bytemuck::cast_slice(&[light]),
      wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    );

    let light_bind_group = gpu::BindGroup::create(&device, &[
      gpu::BindingConfiguration {
        datatype: gpu::ResourceType::Buffer {
          buffer: &light_buffer,
          len: std::mem::size_of_val(&light),
        },
        stage: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
        binding_type: wgpu::BindingType::UniformBuffer {dynamic: false},
      },
    ], Some("light_bind_group"));

    // Camera
    let camera = camera::Camera::new((0.0, 5.0, 10.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
    let projection = camera::Projection::new(sc_desc.width, sc_desc.height, cgmath::Deg(45.0), 0.1, 100.0);
    let camera_controller = camera::CameraController::new(4.0, 0.4);

    // Models
    let (obj_model, cmds) = model::Model::load(&device, "models/cube.obj").unwrap();
    queue.submit(&cmds);

    // Instances
    let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
      (0..NUM_INSTANCES_PER_ROW).map(move |x| {
        let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
        let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

        let position = cgmath::Vector3 {x, y: 0.0, z};

        let rotation = if position.is_zero() {
          cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
        } else {
          cgmath::Quaternion::from_axis_angle(position.clone().normalize(), cgmath::Deg(45.0))
        };

        instance::Instance {position, rotation}
      })
    }).collect::<Vec<_>>();

    let instance_data = instances.iter().map(instance::Instance::to_raw).collect::<Vec<_>>();
    let instance_buffer_size = instance_data.len() * std::mem::size_of::<cgmath::Matrix4<f32>>();
    let instance_buffer = device.create_buffer_with_data(
      bytemuck::cast_slice(&instance_data),
      wgpu::BufferUsage::STORAGE_READ,
    );

    // Uniforms
    let mut uniforms = Uniforms::new();
    uniforms.update_view_proj(&camera, &projection);

    let uniform_buffer = device.create_buffer_with_data(
      bytemuck::cast_slice(&[uniforms]),
      wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
    );

    let uniform_bind_group = gpu::BindGroup::create(&device, &[
      gpu::BindingConfiguration {
        datatype: gpu::ResourceType::Buffer {
          buffer: &uniform_buffer,
          len: std::mem::size_of_val(&uniforms)
        },
        stage: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
        binding_type: wgpu::BindingType::UniformBuffer {
          dynamic: false,
        },
      },
      gpu::BindingConfiguration {
        datatype: gpu::ResourceType::Buffer {
          buffer: &instance_buffer,
          len: instance_buffer_size,
        },
        stage: wgpu::ShaderStage::VERTEX,
        binding_type: wgpu::BindingType::StorageBuffer {
          dynamic: false,
          readonly: true,
        },
      }
    ], Some("uniform_bind_group"));

    // Renderer
    let light_render_pipeline = {
      let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[
          &uniform_bind_group.layout,
          &light_bind_group.layout,
        ]
      });
      
      Ndjinn::create_render_pipeline(
        &device, 
        &layout,
        sc_desc.format,
        Some(texture::Texture::DEPTH_FORMAT), 
        &[model::ModelVertex::desc()],
        &shader::ShaderProgram::create(&device, "src/ndjinn/light.vert", "src/ndjinn/light.frag"),
      )
    };

    let render_pipeline = {
      let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[
          &obj_model.materials[0].bind_group.layout,
          &uniform_bind_group.layout,
          &light_bind_group.layout,
        ],
      });

      Ndjinn::create_render_pipeline(
        &device,
        &layout,
        sc_desc.format,
        Some(texture::Texture::DEPTH_FORMAT),
        &[model::ModelVertex::desc()],
        &shader::ShaderProgram::create(&device, "src/ndjinn/shader.vert", "src/ndjinn/shader.frag"),
      )
    };

    Self {
      surface,
      device,
      queue,
      sc_desc,
      swap_chain,
      depth_texture,
      light,
      light_buffer,
      light_bind_group: light_bind_group.data,
      camera,
      projection,
      camera_controller,
      last_mouse_pos: (0.0, 0.0).into(),
      mouse_pressed: false,
      obj_model,
      instances,
      render_pipeline,
      light_render_pipeline,
      uniforms,
      uniform_buffer,
      uniform_bind_group: uniform_bind_group.data,
      size,
    }
  }

  pub fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    color_format: wgpu::TextureFormat,
    depth_format: Option<wgpu::TextureFormat>,
    vertex_descs: &[wgpu::VertexBufferDescriptor],
    program: &shader::ShaderProgram,
  ) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      layout: &layout,
      vertex_stage: program.vs.stage(Some("main")),
      fragment_stage: Some(program.fs.stage(Some("main"))),
      rasterization_state: Some(wgpu::RasterizationStateDescriptor {
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: wgpu::CullMode::Back,
        depth_bias: 0,
        depth_bias_slope_scale: 0.0,
        depth_bias_clamp: 0.0,
      }),
      primitive_topology: wgpu::PrimitiveTopology::TriangleList,
      color_states: &[
        wgpu::ColorStateDescriptor {
          format: color_format,
          color_blend: wgpu::BlendDescriptor::REPLACE,
          alpha_blend: wgpu::BlendDescriptor::REPLACE,
          write_mask: wgpu::ColorWrite::ALL,
        },
      ],
      depth_stencil_state: depth_format.map(|format| {
        wgpu::DepthStencilStateDescriptor {
          format,
          depth_write_enabled: true,
          depth_compare: wgpu::CompareFunction::Less,
          stencil_front: wgpu::StencilStateFaceDescriptor::IGNORE,
          stencil_back: wgpu::StencilStateFaceDescriptor::IGNORE,
          stencil_read_mask: 0,
          stencil_write_mask: 0,
        }
      }),
      sample_count: 1,
      sample_mask: !0,
      alpha_to_coverage_enabled: false,
      vertex_state: wgpu::VertexStateDescriptor {
        index_format: wgpu::IndexFormat::Uint32,
        vertex_buffers: vertex_descs,
      },
    })
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    self.size = new_size;
    self.sc_desc.width = new_size.width;
    self.sc_desc.height = new_size.height;
    self.projection.resize(new_size.width, new_size.height);
    self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");
    self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
  }

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    match event {
      WindowEvent::KeyboardInput {
        input: KeyboardInput {virtual_keycode: Some(key), state, ..},
        ..
      } => self.camera_controller.process_keyboard(*key, *state),
      WindowEvent::MouseWheel {delta, ..} => {
        self.camera_controller.process_scroll(delta);
        true
      }
      WindowEvent::MouseInput {button: MouseButton::Left, state, ..} => {
        self.mouse_pressed = *state == ElementState::Pressed;
        true
      }
      WindowEvent::CursorMoved {position, ..} => {
        let mouse_dx = position.x as f64 - self.last_mouse_pos.x;
        let mouse_dy = position.y as f64 - self.last_mouse_pos.y;
        self.last_mouse_pos = (position.x, position.y).into();
        if self.mouse_pressed {
          self.camera_controller.process_mouse(mouse_dx, mouse_dy);
        }
        true
      }
      _ => false,
    }
  }

  pub fn update(&mut self, dt: std::time::Duration) {
    self.camera_controller.update_camera(&mut self.camera, dt);
    self.uniforms.update_view_proj(&self.camera, &self.projection);

    let old_pos = self.light.position;
    self.light.position = cgmath::Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(60.0 * dt.as_secs_f32())) * old_pos;

    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("update encoder"),
    });

    let mut staging_buffer = self.device.create_buffer_with_data(
      bytemuck::cast_slice(&[self.uniforms]),
      wgpu::BufferUsage::COPY_SRC,
    );
    encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.uniform_buffer, 0, std::mem::size_of::<Uniforms>() as wgpu::BufferAddress);

    staging_buffer = self.device.create_buffer_with_data(
      bytemuck::cast_slice(&[self.light]),
      wgpu::BufferUsage::COPY_SRC
    );
    encoder.copy_buffer_to_buffer(&staging_buffer, 0, &self.light_buffer, 0, std::mem::size_of::<light::Light>() as wgpu::BufferAddress);

    self.queue.submit(&[encoder.finish()]);
  }

  pub fn render(&mut self) {
    let frame = self.swap_chain.get_next_texture().expect("Timeout getting texture");

    let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("Render Encoder"),
    });

    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      color_attachments: &[
        wgpu::RenderPassColorAttachmentDescriptor {
          attachment: &frame.view,
          resolve_target: None,
          load_op: wgpu::LoadOp::Clear,
          store_op: wgpu::StoreOp::Store,
          clear_color: wgpu::Color {
            r: 0.04,
            g: 0.05,
            b: 0.08,
            a: 1.0,
          },
        }
      ],
      depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
        attachment: &self.depth_texture.view,
        depth_load_op: wgpu::LoadOp::Clear,
        depth_store_op: wgpu::StoreOp::Store,
        clear_depth: 1.0,
        stencil_load_op: wgpu::LoadOp::Clear,
        stencil_store_op: wgpu::StoreOp::Store,
        clear_stencil: 0,
      }),
    });

    render_pass.set_pipeline(&self.light_render_pipeline);
    render_pass.draw_light_model(&self.obj_model, &self.uniform_bind_group, &self.light_bind_group);

    render_pass.set_pipeline(&self.render_pipeline);
    render_pass.draw_model_instanced(&self.obj_model, &self.uniform_bind_group, &self.light_bind_group, 0..self.instances.len() as u32);

    drop(render_pass); // memory management

    self.queue.submit(&[encoder.finish()]);
  }
}
