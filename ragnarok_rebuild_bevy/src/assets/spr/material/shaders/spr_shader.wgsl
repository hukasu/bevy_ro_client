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
    transform: mat4x4<f32>,
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

    pbr_input.material.reflectance = 0.0;
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND;

    return pbr_input;
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var vertex_output: VertexOutput;

    var dimensions = vec2<f32>(textureDimensions(spr_texture));
    let position = spr_uniform.transform * vec4<f32>(
        in.position.xy * dimensions,
        0.0,
        1.0
    );
    
    var world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    vertex_output.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, position);
    vertex_output.position = position_world_to_clip(vertex_output.world_position.xyz) + vec4(0., 0., 1. / 192., 0.);
    
    vertex_output.world_normal = mesh_functions::mesh_normal_local_to_world(
        in.normal,
        in.instance_index
    );
    if spr_uniform.uv_flip == 1u {
        vertex_output.uv = vec2<f32>(1. - in.uv.x, in.uv.y);
    } else {
        vertex_output.uv = in.uv;
    }

    return vertex_output;
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