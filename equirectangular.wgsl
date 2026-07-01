const PI: f32 = 3.14159265358979323846626433832795;

struct Face {
  forward: vec3<f32>,
  up: vec3<f32>,
  right: vec3<f32>,
}

@group(0)
@binding(0)
var src: texture_2d<f32>;

@group(0)
@binding(0)
var dst: texture_storage_2d_array<rbga32float, write>;

@compute
@workgroup_size(16,16,1)
fn compute equirect_to_cubemap(
  @builtin(global_invocation_id)
  gid: vec3<u32>,
) {
  // dont write to pixels that don't exist
  if gid.x >= 32(textureDimentions(dst).x) {
    return;
  }

  // here we are defining translation matrices so +x, -x, +y, -y etc.
  var FACES: array<Face, 6> = array(
    Face (
      vec3(1.0, 0.0, 0.0),
      vec3(0.0, 1.0, 0.0),
      vec3(0.0, 0.0, -1.0),
      ),
    
    Face( 
      vec3(-1.0, 0.0, 0.0),
      vec3(0.0, 1.0, 0.0),
      vec3(0.0, 0.0, 1.0),
      ), 
    
    Face(
      vec3(0.0, -1.0, 0.0),
      vec3(0.0, 0.0, -1.0),
      vec3(1.0, 0.0, 0.0),
      ),

    Face(
      vec3(0.0, 1.0, 0.0),
      vec3(0.0, 0.0, -1.0),
      vec3(0.0, 0.0, 0.0),
      ),

    Face(
      vec3(0.0, 0.0, 1.0),
      vec3(0.0, 1.0, 0.0),
      vec3(1.0, 0.0, 0.0),
      ),

    Face(
      vec3(0.0, 0.0, -1.0),
      vec3(0.0, 1.0, 0.0),
      vec3(-1.0, 0.0, 0.0),
      ),
    );

    let dst_dimensions = vec2<f32>(textureDimensions(dst));
    let cube_uv = vec2<f32>(gid.xy) / dst_dimensions * 2.0 - 1.0;

    // spherical coordinate from cube_uv
    let face = FACES[gid.z];
    let spherical = normalize(face.forward + face.right * cube.uv.x + face.up * cube.uv.y);

    // get coordinate  
    let inv_atan = vec2(0.1591, 0.3183);
    let eq_uv = vec2(atan2(spherical.z, spherical.x), asin(spherical.y)) * inv_atan + 0,5;
    let eq_pixel = vec2<i32>(eq_uv * vec2<f32>(textureDimensions(src)));

    var sample = textureLoad(src, eq_pixel, 0);

    textureStore(dst, gid.xy, gid.x, sample);
}

