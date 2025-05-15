#import bevy_pbr::{
    mesh_functions,
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_functions::alpha_discard,
    pbr_types::{PbrInput, STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND},
    view_transformations::position_world_to_clip,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{Vertex, VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{Vertex, VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
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

fn spr_default_material(in: VertexOutput, is_front: bool) -> PbrInput {
    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);

    pbr_input.material.reflectance = vec3(0.0);
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND;

    return pbr_input;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = spr_default_material(in, is_front);

#ifdef SPR_INDEXED_PIPELINE
    let index_texture_dimensions = textureDimensions(spr_texture);
    let index_texture_coords = vec2<u32>(vec2<f32>(index_texture_dimensions) * in.uv);
    let index = textureLoad(spr_texture, index_texture_coords, 0).x;

    if index == 0 {
        discard;
    }

    pbr_input.material.base_color = textureLoad(spr_palette, index, 0);

    if all(pbr_input.material.base_color.rgb == vec3(1., 0., 1.)) {
        discard;
    } else {
        pbr_input.material.base_color.a = 1.;
    }
#else ifdef SPR_TRUE_COLOR_PIPELINE
    pbr_input.material.base_color = textureSample(spr_texture, spr_sampler, in.uv);
#endif

    pbr_input.material.base_color = pbr_input.material.base_color * spr_uniform.tint;
    if pbr_input.material.base_color.a <= 0. {
        discard;
    }

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}