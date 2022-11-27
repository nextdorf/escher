use egui_winit::{winit::window::Window, egui::{ClippedPrimitive, TexturesDelta, epaint::ahash::HashMap, TextureId}};
use wgpu::{Device, Queue, Surface, TextureViewDescriptor, CommandEncoderDescriptor, RenderPassDescriptor, util::DeviceExt, SurfaceConfiguration};
use std::{ops::FnOnce, num::NonZeroU64};

pub struct WgpuState {
  device: Device,
  queue: Queue,
  surface: Surface,
  surface_config: SurfaceConfiguration,
  textures: HashMap<TextureId, (wgpu::Texture, wgpu::BindGroup)>,

  window_size_bind_group_layout: wgpu::BindGroupLayout,
  window_size_bind_group: Option<wgpu::BindGroup>,
  surface_update_pipeline: wgpu::RenderPipeline,
  surface_update_binding_layout: wgpu::BindGroupLayout,
  surface_scale: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WindowSize {
  pub width: u32,
  pub height: u32,
  pub scale: f32,
  pub __padding: u32,
}

///Describes how wgpu::vertex_buffer are layout for epaint::Vertex
pub fn epaint_vertex_buffer_description<'a>() -> wgpu::VertexBufferLayout<'a> {
  use epaint::Vertex;
  use std::mem::size_of;
  // #[repr(C)]
  // #[derive(Clone, Copy, Debug, Default, PartialEq)]
  // #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
  // #[cfg_attr(feature = "bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
  // pub struct Vertex {
  //     /// Logical pixel coordinates (points).
  //     /// (0,0) is the top left corner of the screen.
  //     pub pos: Pos2, // 64 bit
  
  //     /// Normalized texture coordinates.
  //     /// (0, 0) is the top left corner of the texture.
  //     /// (1, 1) is the bottom right corner of the texture.
  //     pub uv: Pos2, // 64 bit
  
  //     /// sRGBA with premultiplied alpha
  //     pub color: Color32, // 32 bit
  // }
  
  wgpu::VertexBufferLayout {
    array_stride: size_of::<Vertex>() as _,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[
      wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 0,
        shader_location: 0,
      },
      wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: size_of::<[f32;2]>() as _,
        shader_location: 1,
      },
      wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Unorm8x4,
        offset: size_of::<[f32;4]>() as _,
        shader_location: 2,
      },
    ]
  }
}

impl WgpuState {
  pub fn new(window: &Window, surface_scale: f32) -> Option<Self> {
    let (device, queue, surface, surface_config) = Self::setup_wgpu(window)?;
    let (surface_update_pipeline, surface_update_binding_layout, window_size_bind_group_layout) =
      Self::create_surface_pipeline(&device, &surface_config);

    Some(Self {
      device,
      queue,
      surface,
      surface_config,
      textures: HashMap::default(),

      window_size_bind_group_layout,
      window_size_bind_group: None,
      surface_update_pipeline,
      surface_update_binding_layout,
      surface_scale,
    })
  }

  fn create_window_size_bind_group(&self) -> wgpu::BindGroup {
    WindowSize::new(self.surface_config.width, self.surface_config.height, self.surface_scale)
      .get_bind_group(&self.device, &self.window_size_bind_group_layout)
  }

  pub fn update_window_size_bind_group(&mut self, and_invalidate: bool) {
    if and_invalidate || self.window_size_bind_group.is_none() {
      self.window_size_bind_group = Some(self.create_window_size_bind_group())
    }
  }

  pub fn invalidate_window_size_bind_group(&mut self) {
    self.window_size_bind_group = None;
  }


  pub fn get_surface_scale(&self) -> f32 { self.surface_scale }

  fn setup_wgpu(window: &Window) -> Option<(Device, Queue, Surface, SurfaceConfiguration)> {
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = pollster::block_on(instance.request_adapter(
      &wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }))?;

    let (device, queue) = pollster::block_on(adapter.request_device(
      &wgpu::DeviceDescriptor {
        label: Some("device"),
        features: wgpu::Features::default(),
        limits: wgpu::Limits::default(),
      },
      None,
    )).ok()?;


    let size = window.inner_size();
    let all_surface_formats = surface.get_supported_formats(&adapter);
    eprintln!("all_surface_formats:");
    for fmt in all_surface_formats.iter() {
      eprintln!("- {:?}", fmt);
    }
    eprintln!("");
    let surface_format = all_surface_formats[0];
    let surface_config = SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format: surface_format,
      width: size.width.max(1),
      height: size.height.max(1),
      present_mode: wgpu::PresentMode::Fifo,
      alpha_mode: wgpu::CompositeAlphaMode::Auto,
    };

    surface.configure(&device, &surface_config);

    Some((device, queue, surface, surface_config))
  }

  fn create_surface_pipeline(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration)
    -> (wgpu::RenderPipeline, wgpu::BindGroupLayout, wgpu::BindGroupLayout)
  {
    //shader
    let shader = device.create_shader_module(
      wgpu::ShaderModuleDescriptor {
        label: Some("egui.wgsl"),
        source: wgpu::ShaderSource::Wgsl(include_str!("egui.wgsl").into()),
    });

    //Binding_layout
    let texture_bind_group_layout = device.create_bind_group_layout(
      &wgpu::BindGroupLayoutDescriptor {
        label: Some("surface_textures_bind_group_layout"),
        entries: &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
              sample_type: wgpu::TextureSampleType::Float { filterable: true },
              view_dimension: wgpu::TextureViewDimension::D2,
              multisampled: false
            },
            count: None,
          },
          wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
          },
        ],
    });

    let window_size_bind_group_layout = WindowSize::get_bind_group_layout(&device);

    //The actual pipeline
    let pipeline_layout = device.create_pipeline_layout(
      &wgpu::PipelineLayoutDescriptor {
        label: Some("surface_update_pipeline_descriptor"),
        bind_group_layouts: &[
          &texture_bind_group_layout,
          &window_size_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(
      &wgpu::RenderPipelineDescriptor {
        label: Some("surface_update_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
          module: &shader,
          entry_point: "main_vs",
          buffers: &[ epaint_vertex_buffer_description() ]
        },
        primitive: wgpu::PrimitiveState {
          topology: wgpu::PrimitiveTopology::TriangleList,
          strip_index_format: None,
          front_face: wgpu::FrontFace::Cw,
          cull_mode: None,
          unclipped_depth: false,
          polygon_mode: wgpu::PolygonMode::Fill,
          conservative: false
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
          count: 1,
          mask: !0,
          alpha_to_coverage_enabled: false, //Wichtig damit fonts sauber gerendert werden?
        },
        fragment: Some(wgpu::FragmentState {
          module: &shader,
          entry_point: "main_fs",
          targets: &[Some(wgpu::ColorTargetState {
            format: surface_config.format,
            blend: Some(wgpu::BlendState {
              color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add
              },
              alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add
              },
            }),
            write_mask: wgpu::ColorWrites::ALL,
          })],
        }),
        multiview: None,
    });
    (pipeline, texture_bind_group_layout, window_size_bind_group_layout)
  }

  fn new_surface_update_binding(&self, texture_id: &TextureId, texture: &wgpu::Texture, tex_filter: wgpu::FilterMode) -> wgpu::BindGroup {
    self.device.create_bind_group(
      &wgpu::BindGroupDescriptor {
        label: Some(format!("surface_texture_bind_group {:?}", texture_id).as_str()),
        layout: &self.surface_update_binding_layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(
              &texture.create_view(&wgpu::TextureViewDescriptor::default())),
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(
              &self.device.create_sampler(&wgpu::SamplerDescriptor{
                mag_filter: tex_filter,
                min_filter: tex_filter,
                // mipmap_filter: tex_filter,
                ..wgpu::SamplerDescriptor::default()
            })),
          },
        ],
    })
  }

  fn redraw_alloc_new_textures(&self, texture_delta_set: &Vec<(TextureId, epaint::ImageDelta)>) -> Vec<(TextureId, wgpu::Texture, wgpu::BindGroup)> {
    use egui_winit::egui;
    let font_gamma = 1.;

    let mut res = Vec::with_capacity(texture_delta_set.len());
    for (texture_id, img_delta) in texture_delta_set.iter() {
      if img_delta.pos.is_some() {
        //FIXME
        eprintln!("Not sure where to place {:?}...", dbg!(texture_id));
      }
      let pixel_data;
      let pixel_data_store: Vec<_>;
      let rgba8_to_surface_format = if self.surface_config.format.describe().srgb {
          wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
          wgpu::TextureFormat::Rgba8Unorm
        };
      pixel_data = match &img_delta.image {
        egui::ImageData::Color(img) => bytemuck::cast_slice(img.pixels.as_slice()),
        egui::ImageData::Font(img) => {
          pixel_data_store = img.pixels.iter().flat_map(|gamma| {
            let val = (gamma.powf(font_gamma/2.2)*255.).round() as _;
            [val, val, val, val]
          }).collect();
          pixel_data_store.as_slice()
        },
      };

      //Expects unmultiplied RGBA and wgsl will see unmultiplied sRGBA
      let tex = self.device.create_texture_with_data(
        &self.queue,
        &wgpu::TextureDescriptor {
          label: Some(format!("Texture {:?}", texture_id).as_str()),
          size: wgpu::Extent3d {
            width: img_delta.image.width() as _,
            height: img_delta.image.height() as _,
            depth_or_array_layers: 1
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: rgba8_to_surface_format,
          usage: wgpu::TextureUsages::TEXTURE_BINDING //COPY_DST is added automatically
        },
        pixel_data
      );

      let tex_filter = match img_delta.filter {
        egui::TextureFilter::Nearest => wgpu::FilterMode::Nearest,
        egui::TextureFilter::Linear => wgpu::FilterMode::Linear,
      };
      let tex_binding = self.new_surface_update_binding(texture_id, &tex, tex_filter);

      // self.textures.insert(texture_id.clone(), (tex, tex_binding));
      res.push((texture_id.clone(), tex, tex_binding));
    }

    res

  }

  pub fn redraw(&mut self, f: impl FnOnce() -> (TexturesDelta, Vec<ClippedPrimitive>)) -> Option<()> {
    use egui_winit::egui::epaint::Primitive;
    let current_frame = self.surface.get_current_texture().ok()?;
    let (texture_delta, paint_jobs) = f();

    let window_size_bind_group_store;
    let window_size_bind_group = match self.window_size_bind_group.as_ref() {
      Some(bind_group) => bind_group,
      None => {
          window_size_bind_group_store = self.create_window_size_bind_group();
          &window_size_bind_group_store
        }
    };

    //Alloc new textures
    let new_textures = self.redraw_alloc_new_textures(&texture_delta.set);
    for (id, tex, tex_binding) in new_textures {
      self.textures.insert(id, (tex, tex_binding));
    }

    //Render deltas
    let current_view = current_frame.texture.create_view(
      &TextureViewDescriptor::default());
    let mut encoder = self.device.create_command_encoder(
      &CommandEncoderDescriptor { label: Some("current_frame_redraw_encoder") });
    let mut vertex_buffers = Vec::with_capacity(paint_jobs.len());
    let mut vert_inds_buffers = Vec::with_capacity(paint_jobs.len());

    for ClippedPrimitive { clip_rect, primitive } in paint_jobs.iter() {
      let mut render_pass = encoder.begin_render_pass(
        &RenderPassDescriptor {
          label: Some("current_frame_redraw_render_pass"),
          color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &current_view,
            resolve_target: None,
            ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: true }
          })],
          depth_stencil_attachment: None,
      });

      let mesh = match primitive {
        Primitive::Mesh(m) => m,
        Primitive::Callback(_) => {eprintln!("Callback"); return None}, //TODO: not done yet
      };
      let (_texture, bind_group) = self.textures.get(&mesh.texture_id)?;
      

      vertex_buffers.push(self.device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
          label: Some("vertex_buffer for egui"),
          contents: bytemuck::cast_slice(mesh.vertices.as_slice()),
          usage: wgpu::BufferUsages::VERTEX,
      }));
      let vertex_buffer = vertex_buffers.last().unwrap();

      vert_inds_buffers.push(self.device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
          label: Some("vertex_index_buffer for egui"),
          contents: bytemuck::cast_slice(mesh.indices.as_slice()),
          usage: wgpu::BufferUsages::INDEX,
      }));
      let vert_inds_buffer = vert_inds_buffers.last().unwrap();

      // let clip_rect_top_left = ((clip_rect.min.x*self.get_surface_scale()) as _, (clip_rect.min.y*self.get_surface_scale()) as _);
      // let clip_rect_bottom_right: (u32, u32) = ((clip_rect.max.x*self.get_surface_scale()) as _, (clip_rect.max.y*self.get_surface_scale()) as _);
      // let clip_rect_size = (clip_rect_bottom_right.0 - clip_rect_top_left.0, clip_rect_bottom_right.1 - clip_rect_top_left.1);
      // let full_size = (self.surface_config.width, self.surface_config.height);

      render_pass.set_pipeline(&self.surface_update_pipeline);
      // if (clip_rect_bottom_right.0 < full_size.0 && clip_rect_bottom_right.1 <= full_size.1) || (clip_rect_bottom_right.0 <= full_size.0 && clip_rect_bottom_right.1 < full_size.1) {
      //   render_pass.set_scissor_rect(clip_rect_top_left.0, clip_rect_top_left.1, clip_rect_size.0, clip_rect_size.1);
      //   eprintln!("scissor_rect: {:?}", (clip_rect_top_left.0, clip_rect_top_left.1, clip_rect_size.0, clip_rect_size.1));
      // }
      render_pass.set_bind_group(0, bind_group, &[]);
      render_pass.set_bind_group(1, &window_size_bind_group, &[]);
      render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
      render_pass.set_index_buffer(vert_inds_buffer.slice(..), wgpu::IndexFormat::Uint32);
      render_pass.draw_indexed(0..mesh.indices.len() as _, 0, 0..1);
    }

    //submit commands
    self.queue.submit(std::iter::once(encoder.finish()));
    current_frame.present();


    //Destroy textures
    for texture_id in texture_delta.free.iter() {
      if let Some((tex, _)) = self.textures.remove(texture_id) {
        tex.destroy();
      }
    }

    Some(())
  }

  pub fn resize(&mut self, width: Option<u32>, height: Option<u32>, scale: Option<f32>, win_state: &mut egui_winit::State) {
    if width.is_none() && height.is_none() && scale.is_none() {
      return;
    }
    self.invalidate_window_size_bind_group();

    if let Some(w) = width {
      self.surface_config.width = w.max(1);
    }
    if let Some(h) = height {
      self.surface_config.height = h.max(1);
    }
    if let Some(s) = scale {
      let new_scale = s.max(0.1);
      self.surface_scale = new_scale;
      win_state.set_pixels_per_point(new_scale);
    }

    self.surface.configure(&self.device, &self.surface_config);
  }
}

impl WindowSize {
  pub fn new(width: u32, height: u32, scale: f32) -> Self {
    Self {
      width,
      height,
      scale,
      __padding: 0
    }
  }

  fn bind_size() -> Option<NonZeroU64> {
    NonZeroU64::new(std::mem::size_of::<WindowSize>() as _)
  }

  pub fn get_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(
      &wgpu::BindGroupLayoutDescriptor {
        label: Some("window_size_bind_group_layout"),
        entries: &[
          wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
              ty: wgpu::BufferBindingType::Uniform,
              has_dynamic_offset: false,
              min_binding_size: Self::bind_size()
            },
            count: None,
          },
        ],
    })
  }

  pub fn get_bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
    let buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("window_size_buffer"),
        contents: bytemuck::bytes_of(self),
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::UNIFORM
    });

    device.create_bind_group(
      &wgpu::BindGroupDescriptor {
        label: Some("window_size_bind_group"),
        layout: &layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
              buffer: &buffer,
              offset: 0,
              size: Self::bind_size()
            }),
        }]
    })
  }
}




