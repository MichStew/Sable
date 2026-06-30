// I suppose we need the camera here to check it against what would be 'normal' to the cameras 'eye'
struct Camera {
  view_pos: mat4x4<f32>,
  view: mat4x4<f32>,
  view_proj: mat4x4<f32>,
  inv_proj: mat4x4<f32>,
  inv_view: mat4x4<f32>,
};

// in the light pipeline, expect some data about the camera. 
@group(0) @binding(0)
var<uniform> camera: Camera;

struct Light {
  position: vec3<f32>,
  color: vec3<f32>,
}; 

// in the light pipeline, expect some data about the light
@group(1) @binding(0)
var<uniform> light: Light;

struct VertexInput {
  @location(0) position: vec3<f32>,
};

// we are going to do some operations in the vertex shader 
struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
  model: VertexInput,
) -> VertexOutput {
  let scale = 0.25;
  var out: VertexOutput;
// clip the light that is not visible to the camera.
// this is like saying what is the camera projections view on the position of the model that has a scale and a light position on it. 
// the underlying math is still fuzzy but the camer proj matrix will end up calculating a 2d pixel
// need more intuition in this area
  out.clip_position = camera.view_proj * vec4<f32>(model.position * scale + light.position, 1.0);
  out.color = light.color;
  return out;
}

// dont change the color lol. 
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> { 
  return vec4<f32>(in.color, 1.0);
}

