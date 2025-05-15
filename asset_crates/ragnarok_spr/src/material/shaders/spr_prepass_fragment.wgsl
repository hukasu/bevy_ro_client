#import bevy_pbr::prepass_io::VertexOutput
#ifdef PREPASS_FRAGMENT
#import bevy_pbr::prepass_io::FragmentOutput
#endif

struct SprUniform {
    uv_flip: u32,
    tint: vec4<f32>,
}

@group(2) @binding(0) var<uniform> spr_uniform: SprUniform;

#ifdef SPR_INDEXED_PIPELINE
@group(2) @binding(1) var spr_texture: texture_2d<u32>;
@group(2) @binding(2) var spr_palette: texture_1d<f32>;
#else ifdef SPR_TRUE_COLOR_PIPELINE
@group(2) @binding(1) var spr_texture: texture_2d<f32>;
@group(2) @binding(2) var spr_sampler: sampler;
#endif

@fragment
fn fragment(
    in: VertexOutput,
#ifdef PREPASS_FRAGMENT
) -> FragmentOutput {
#else 
) {
#endif
    var color: vec4<f32>;
#ifdef SPR_INDEXED_PIPELINE
    let index_texture_dimensions = textureDimensions(spr_texture);
    let index_texture_coords = vec2<u32>(vec2<f32>(index_texture_dimensions) * in.uv);
    let index = textureLoad(spr_texture, index_texture_coords, 0).x;

    if index == 0 {
        discard;
    }

    color = textureLoad(spr_palette, index, 0);

    if all(color.rgb == vec3(1., 0., 1.)) {
        discard;
    }
#else ifdef SPR_TRUE_COLOR_PIPELINE
    color = textureSample(spr_texture, spr_sampler, in.uv);
#endif

#ifdef PREPASS_FRAGMENT
    var out: FragmentOutput;
#ifdef DEPTH_CLAMP_ORTHO
    out.frag_depth = in.clip_position_unclamped.z;
#endif
    return out;
#endif
}