// Shaders

@location(1) struct Sprite {
    pos_basis: vec2<f32>,
    pos_size: vec2<f32>,
    tex_basis: vec2<f32>,
    pos_basis: vec2<f32>
};

struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(0)
@binding(0)
var gray: texture_2d<f32>;

@group(0)
@binding(1)
var mod_wheel: f32;

// Quad positioning
@vertex
fn vs_quad(
    @location(0) position: vec2<f32>
) -> VertexOutput {
    var result: VertexOutput;
    result.tex_coord = position;
    let full_position = position*2. - vec2(1.,1.);
    result.position = vec4(full_position.x, -full_position.y, 0., 1.);
    return result;
}

// Draw quad unaltered (for debug?)
@fragment
fn fs_quad_direct(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let dim = textureDimensions(gray);
    let tex = textureLoad(gray, vec2<u32>(vertex.tex_coord*vec2<f32>(dim)), 0);
    let v = f32(tex.x); //  / 255.0
    return vec4<f32>(tex.rrr, 1.0);
}

// Draw quad unaltered (for debug?)
@fragment
fn fs_quad_threshold(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let dim = textureDimensions(gray);
    let tex = textureLoad(gray, vec2<u32>(vertex.tex_coord*vec2<f32>(dim)), 0);
    let v = f32(tex.x); //  / 255.0
    return vec4<f32>(tex.rrr, 1.0);
}

// Draw quad dark green
@fragment
fn fs_green(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 1.0);
}
