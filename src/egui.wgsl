// epaint::mesh::Vertex
// pub struct Vertex {
//   /// Logical pixel coordinates (points).
//   /// (0,0) is the top left corner of the screen.
//   pub pos: Pos2, // 64 bit
//
//   /// Normalized texture coordinates.
//   /// (0, 0) is the top left corner of the texture.
//   /// (1, 1) is the bottom right corner of the texture.
//   pub uv: Pos2, // 64 bit
//
//   /// sRGBA with premultiplied alpha
//   pub color: Color32, // 32 bit
// }
struct Vertex {                   // 32 bytes
  @location(0) pos: vec2<f32>,    // 8 bytes
  @location(1) uv: vec2<f32>,     // 8 bytes
  @location(2) color: vec4<f32>,  // 16 bytes, rgba8, see also https://gpuweb.github.io/gpuweb/wgsl/#channel-formats
  //No padding
}

// #[repr(C)]
// #[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct WindowSize {
//   pub width: u32,
//   pub height: u32,
//   pub scale: f32,
// }
struct WindowSize {             // 12 + 4 = 16 bytes
  @location(0) size: vec2<u32>, // 8 bytes
  @location(1) scale: f32,      // 4 bytes
  // implicit padding of 4 bytes
}


// struct VertexInput {
//   @location(0) pos: vec3<f32>,
//   @location(1) uv: vec2<f32>,
// };

struct VertexOutput {
  @builtin(position) pos: vec4<f32>,
  @location(0) color: vec4<f32>,
  @location(1) uv: vec2<f32>,
};


// Vertex shader

@group(1) @binding(0)
var<uniform> window_size: WindowSize;

@vertex
fn main_vs(vert: Vertex) -> VertexOutput {
  var res: VertexOutput;
  let rescaled_size = vert.pos/vec2<f32>(window_size.size)*window_size.scale;
  res.pos = vec4(
    rescaled_size.x*2. - 1.,
    -rescaled_size.y*2. + 1.,
    0.,
    1.);
  // res.color = vec4<f32>(vert.color.rgb/vert.color.a, vert.color.a);
  res.color = vert.color;
  res.uv = vert.uv;
  // res.uv = vec2<i32>(round(vert.pos));
  return res;
}


// Fragment shader

@group(0) @binding(0)
var egui_texture: texture_2d<f32>;
@group(0) @binding(1)
var egui_sampler: sampler;

@fragment
fn main_fs(vert: VertexOutput) -> @location(0) vec4<f32> {
  return vert.color*textureSample(egui_texture, egui_sampler, vert.uv);
  // let mip_level = 0;
  // //Cant use u32, see https://github.com/gfx-rs/naga/issues/1997 (naga-bug)
  // // var coord = vec2<i32>(round(vert.uv * vec2<f32>((textureDimensions(egui_texture, mip_level)))));
  // var coord = vec2<i32>(vert.uv * vec2<f32>((textureDimensions(egui_texture, mip_level)))); 
  // return vert.color*textureLoad(egui_texture, coord, mip_level);
}

