pub struct Shader {
  pub id: wgpu::ShaderStage,
  pub module: wgpu::ShaderModule,
}

impl Shader {
  pub fn stage<'a>(&'a self, entry: Option<&'a str>) -> wgpu::ProgrammableStageDescriptor<'a> {
    wgpu::ProgrammableStageDescriptor {
      module: &self.module,
      entry_point: entry.or(Some("main")).unwrap(),
    }
  }
}

pub struct ShaderProgram {
  pub vs: Shader,
  pub fs: Shader,
}

impl ShaderProgram {
  /// Creates a new shader program from vertex and fragment shader file paths.
  pub fn create(device: &wgpu::Device, vs_file_name: &str, fs_file_name: &str) -> Self {
    // Parse
    let vs_src = std::fs::read_to_string(vs_file_name).unwrap();
    let fs_src = std::fs::read_to_string(fs_file_name).unwrap();

    // Compile
    let mut compiler = shaderc::Compiler::new().unwrap();
    let vs_spirv = compiler.compile_into_spirv(&vs_src, shaderc::ShaderKind::Vertex, vs_file_name, "main", None).unwrap();
    let fs_spirv = compiler.compile_into_spirv(&fs_src, shaderc::ShaderKind::Fragment, fs_file_name, "main", None).unwrap();

    // Read
    let vs_data = wgpu::read_spirv(std::io::Cursor::new(vs_spirv.as_binary_u8())).unwrap();
    let fs_data = wgpu::read_spirv(std::io::Cursor::new(fs_spirv.as_binary_u8())).unwrap();

    let vs = Shader {
      id: wgpu::ShaderStage::VERTEX,
      module: device.create_shader_module(&vs_data),
    };

    let fs = Shader {
      id: wgpu::ShaderStage::FRAGMENT,
      module: device.create_shader_module(&fs_data),
    };

    Self {vs, fs}
  }
}