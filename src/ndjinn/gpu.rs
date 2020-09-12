pub struct BindGroup {
  pub layout: wgpu::BindGroupLayout,
  pub data: wgpu::BindGroup,
}

pub struct BindingConfiguration<'a> {
  pub datatype: ResourceType<'a>,
  pub stage: wgpu::ShaderStage,
  pub binding_type: wgpu::BindingType,
}

#[allow(dead_code)]
pub enum ResourceType<'a> {
  Buffer {
    buffer: &'a wgpu::Buffer,
    len: usize,
  },
  Texture {
    view: &'a wgpu::TextureView,
  },
  Sampler {
    sampler: &'a wgpu::Sampler,
  },
}

impl BindGroup {
  fn map_buffer_resource<'a>(index: u32, config: &'a BindingConfiguration<'a>, buffer: &'a wgpu::Buffer, len: usize)
  -> (wgpu::Binding<'a>, wgpu::BindGroupLayoutEntry) {
    let binding = wgpu::Binding {
      binding: index,
      resource: wgpu::BindingResource::Buffer {
        buffer,
        range: 0..len as wgpu::BufferAddress,
      },
    };
    let layout = wgpu::BindGroupLayoutEntry {
      binding: index,
      visibility: config.stage,
      ty: config.binding_type,
    };

    (binding, layout)
  }

  fn map_texture_resource<'a>(index: u32, config: &'a BindingConfiguration<'a>, view: &'a wgpu::TextureView)
  -> (wgpu::Binding<'a>, wgpu::BindGroupLayoutEntry) {
    let binding = wgpu::Binding {
      binding: index,
      resource: wgpu::BindingResource::TextureView(view),
    };
    let layout = wgpu::BindGroupLayoutEntry {
      binding: index,
      visibility: config.stage,
      ty: config.binding_type,
    };

    (binding, layout)
  }

  fn map_sampler_resource<'a>(index: u32, config: &'a BindingConfiguration<'a>, sampler: &'a wgpu::Sampler)
  -> (wgpu::Binding<'a>, wgpu::BindGroupLayoutEntry) {
    let binding = wgpu::Binding {
      binding: index,
      resource: wgpu::BindingResource::Sampler(sampler),
    };
    let layout = wgpu::BindGroupLayoutEntry {
      binding: index,
      visibility: config.stage,
      ty: config.binding_type,
    };

    (binding, layout)
  }

  pub fn create<'a>(device: &wgpu::Device, config: &'a [BindingConfiguration<'a>], label: Option<&'a str>) -> Self {
    let mut layouts: Vec<wgpu::BindGroupLayoutEntry> = vec![];
    let mut bindings: Vec<wgpu::Binding> = vec![];

    for (i, c) in config.into_iter().enumerate() {
      match c.datatype {
        ResourceType::Buffer {buffer, len} => {
          let (binding, layout) = BindGroup::map_buffer_resource(i as u32, &c, &buffer, len);
          bindings.push(binding);
          layouts.push(layout);
        },
        ResourceType::Texture {view} => {
          let (binding, layout) = BindGroup::map_texture_resource(i as u32, &c, &view);
          bindings.push(binding);
          layouts.push(layout);
        }
        ResourceType::Sampler {sampler} => {
          let (binding, layout) = BindGroup::map_sampler_resource(i as u32, &c, &sampler);
          bindings.push(binding);
          layouts.push(layout);
        },
      }
    }

    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      bindings: &layouts,
      label: Some(&[label.unwrap_or("bind_group"), "_layout"].concat()),
    });

    let data = device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: &layout,
      bindings: &bindings,
      label: label.or(Some("bind_group")),
    });

    Self {data, layout}
  }
}