//vertex shader... my first!
struct VertexOutput {
	@builtin(position) clip_position: vec4<f32>,
	//@location(0) vert_pos: vec3<f32>,
};

// this took me so long to figure out
// so when I was using positive numbers I was getting nothing - or an entire brown screen
// using negative y components causes the triangle to face the screen, so it doesn't get culled (and disappear)
// I will have to play around a bit more becau
const TRI_VERTICES = array(
vec4(0.,0.5,0.,1.),
vec4(-0.5,-0.2,0.,1.),
vec4(0.5,-0.2,0.,1.),
);


@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
return TRI_VERTICES[in_vertex_index];
}




/*
fn test(
	@builtin(vertex_index) in_vertex_index: u32,
	) -> VertexOutput {
		var out: VertexOutput;
		let x = 0.2;
		let y = -0.3;
		out.clip_position = vec4<f32>(x, y, 0.0, 0.1);
		return out; 
	}

*/	
// the syntax is a bit weird here 
// so we declare a struct which will contain a vector of length 4 of 32 bit floats
// @builtin(position) tells wgpu to use these coordinates as clip coordinates ( assume when the unit cube comes into play
// for clipping.
// we are using @vertex to mark this function  as a valid entry point for the vertex position
// this function expects a vertex index...as a param?(technically yes i think) 
// declare out var of struct vertexoutput, set variables and return via our.clip positon...return out

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4(0.3, 0.2, 0.1, 1.0);
	}

// above function sets the current fragment to brown...
// location(0) is the 'first color target'

