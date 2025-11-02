#import bevy_pbr::{
    mesh_functions,
    mesh_bindings::mesh,
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_types::{PbrInput, STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND},
    view_transformations::position_world_to_clip,
}

#ifdef MESH_PIPELINE
#import bevy_pbr::{
    forward_io::{Vertex, VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#elseif PREPASS_PIPELINE
#import bevy_pbr::prepass_io::{
    Vertex, VertexOutput, FragmentOutput
}
#endif

struct SprUvs {
    min: vec2<f32>,
    max: vec2<f32>,
}

#ifdef BINDLESS

#else // BINDLESS

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var spr_indexed_texture: texture_2d<u32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var spr_palette: texture_1d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var spr_true_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var spr_true_color_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var<storage> spr_indexed_uvs: array<SprUvs>;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<storage> spr_true_color_uvs: array<SprUvs>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var<storage> spr_uv_index: array<u32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var<storage> spr_tint: array<vec4<f32>>;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var<storage> spr_flipped_uvs: array<u32>;

#endif // BINDLESS

#ifdef MESH_PIPELINE
fn spr_default_material(in: VertexOutput, is_front: bool) -> PbrInput {
    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);

    pbr_input.material.reflectance = vec3(0.0);
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND;

    return pbr_input;
}
#endif // MESH_PIPELINE

@vertex
fn vertex(
    in: Vertex,
#ifndef MORPH_TARGETS
    @builtin(vertex_index) vertex_index: u32,
#endif    
) -> VertexOutput {
    var vertex_output: VertexOutput;

    let tag = mesh[in.instance_index].tag;
    let first_vertex_index = mesh[in.instance_index].first_vertex_index;
#ifndef MORPH_TARGETS
    let index = vertex_index - first_vertex_index;
#else
    let index = in.vertex_index - first_vertex_index;
#endif

    let uv_index = spr_uv_index[tag] & 0x70000000;

    var dimensions: vec2<f32>;
    var spr_uv: SprUvs;
    if (spr_uv_index[tag] & 0x80000000) != 0 {
        dimensions = vec2<f32>(textureDimensions(spr_true_color_texture));
        spr_uv = spr_true_color_uvs[uv_index];
    } else {
        dimensions = vec2<f32>(textureDimensions(spr_indexed_texture));
        spr_uv = spr_indexed_uvs[uv_index];
    }

    var uv: vec2<f32> = in.uv;
    if spr_flipped_uvs[tag] == 1u {
        uv = vec2<f32>(1. - uv.x, uv.y);
    }

    let uv_dimensions = spr_uv.max - spr_uv.min;
    vertex_output.uv = spr_uv.min + uv * uv_dimensions;

    let position = vec4<f32>(
        in.position.xy * dimensions * uv_dimensions,
        0.0,
        1.0
    );
    
    var world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    vertex_output.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, position);
    vertex_output.position = position_world_to_clip(vertex_output.world_position.xyz) +  vec4(0., 0., 1. / 192., 0.);

#ifdef VERTEX_NORMALS || NORMAL_PREPASS_OR_DEFERRED_PREPASS
    vertex_output.world_normal = mesh_functions::mesh_normal_local_to_world(
        in.normal,
        in.instance_index
    );
#endif

    vertex_output.instance_index = in.instance_index;

    return vertex_output;
}

#ifdef MESH_PIPELINE
@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool
) -> FragmentOutput {
    var pbr_input = spr_default_material(in, is_front);
    pbr_input.material.base_color = vec4(1.);

    let tag = mesh[in.instance_index].tag;
    let uv_index = spr_uv_index[tag] & 0x70000000;

    if (spr_uv_index[tag] & 0x80000000) != 0 {
        pbr_input.material.base_color = textureSample(
            spr_true_color_texture,
            spr_true_color_sampler,
            in.uv
        );
    } else {
        let spr_uv = spr_indexed_uvs[uv_index];
        let dimensions = vec2<f32>(textureDimensions(spr_indexed_texture));
        let coord = vec2<u32>(dimensions * in.uv);
        let index = textureLoad(spr_indexed_texture, coord, 0).x;

        if index == 0 {
            discard;
        }

        pbr_input.material.base_color = textureLoad(spr_palette, index, 0);

        if all(pbr_input.material.base_color.rgb == vec3(1., 0., 1.)) {
            discard;
        } else {
            pbr_input.material.base_color.a = 1.;
        }
    }

    let tint = spr_tint[tag];

    pbr_input.material.base_color = pbr_input.material.base_color * tint;
    if pbr_input.material.base_color.a <= 0. {
        discard;
    }

    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}
#elseif PREPASS_PIPELINE
#ifdef PREPASS_FRAGMENT
@fragment
fn fragment() -> FragmentOutput {
    var out: FragmentOutput;

    return out;
}
#else // PREPASS_FRAGMENT
@fragment
fn fragment() {
    return;
}
#endif // PREPASS_FRAGMENT
#endif // MESH_PIPELINE
