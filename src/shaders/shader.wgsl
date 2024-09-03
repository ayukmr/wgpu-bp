struct VertexInput {
  @builtin(vertex_index)
  vtx_idx: u32,
}

struct VertexOutput {
  @builtin(position)
  clip_pos: vec4f,

  @location(0)
  color: vec4f,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
  var out: VertexOutput;

  var pos = array(
    vec2( 0.0,  0.5),
    vec2(-0.5, -0.5),
    vec2( 0.5, -0.5),
  );

  var color = array(
    vec3f(1.0, 0.0, 0.0),
    vec3f(0.0, 1.0, 0.0),
    vec3f(0.0, 0.0, 1.0),
  );

  out.clip_pos = vec4f(pos[in.vtx_idx], 0.0, 1.0);
  out.color    = vec4f(color[in.vtx_idx], 1.0);

  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
  return in.color;
}
