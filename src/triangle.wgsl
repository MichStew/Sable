@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
	let x = f32(i32(in_vertex_index) -1);
// &1u only lets 1 through, I guess 1 and zero, this is a bit overkill for one triangle, and only calling this a few times lol. 
// it really just clamps odd numbers to 1 or -1 right? and then the x coordinate is just going to increase forever.
	let y = f32(i32(in_vertex_index & 1u) *2 -1);
	return vec4<f32>(x,y,0.0,1.0); // the fourth number here is a scaling factor - RRR
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
	return vec4<f32>(1.0,0.0,0.0,1.0); // regular color for this triangle. RGBA I think.
	}
