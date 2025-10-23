#import bevy_pbr::{
    mesh_functions,
    mesh_bindings::mesh,
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_types::{PbrInput, STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK},
    view_transformations::position_world_to_clip,
    pbr_functions::alpha_discard,
}
#ifdef BINDLESS
#import bevy_render::bindless::{bindless_samplers_filtering, bindless_textures_2d}
#endif  // BINDLESS

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

struct GndCubeFace {
    bottom_left: f32,
    bottom_right: f32,
    top_left: f32,
    top_right: f32,
    surface_id: u32,
}

struct GndSurface {
    bottom_left_uv: vec2<f32>,
    bottom_right_uv: vec2<f32>,
    top_left_uv: vec2<f32>,
    top_right_uv: vec2<f32>,
}

#ifdef BINDLESS

struct GndBindings {
    cube_face: u32,
    surface: u32,
    texture: u32,
    texture_sampler: u32,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<storage> gnd_bindings: array<GndBindings>;
@group(#{MATERIAL_BIND_GROUP}) @binding(10) var<storage> gnd_cube_faces: array<GndCubeFace>;
@group(#{MATERIAL_BIND_GROUP}) @binding(11) var<storage> gnd_surfaces: array<GndSurface>;

#else // BINDLESS

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<storage> gnd_cube_face: GndCubeFace;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage> gnd_surface: array<GndSurface>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var gnd_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var gnd_texture_sampler: sampler;

#endif // BINDLESS

#ifdef MESH_PIPELINE
fn gnd_default_material(in: VertexOutput, is_front: bool) -> PbrInput {
    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);

    pbr_input.material.reflectance = vec3(0.04);
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK;

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

    let first_vertex_index = mesh[in.instance_index].first_vertex_index;
#ifdef BINDLESS
    let slot = mesh[in.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
    let cube_face = gnd_cube_faces[gnd_bindings[slot].cube_face];
    let surface = gnd_surfaces[gnd_bindings[slot].surface + cube_face.surface_id];
#else // BINDLESS
    let cube_face = gnd_cube_face;
    let surface = gnd_surface[surface_id];
#endif // BINDLESS

#ifdef MORPH_TARGETS
    let index = in.index - first_vertex_index;
#else
    let index = vertex_index - first_vertex_index;
#endif

    var y: f32;
    if index == 0 {
        y = cube_face.bottom_right;
    } else if index == 1 {
        y = cube_face.bottom_left;
    } else if index == 2 {
        y = cube_face.top_right;
    } else if index == 3 {
        y = cube_face.top_left;
    }

    let position = vec4<f32>(
        in.position.x,
        y,
        in.position.z,
        1.0
    );
    
    var world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    vertex_output.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, position);
    vertex_output.position = position_world_to_clip(vertex_output.world_position.xyz);
    
#ifdef VERTEX_NORMALS || NORMAL_PREPASS_OR_DEFERRED_PREPASS
    vertex_output.world_normal = mesh_functions::mesh_normal_local_to_world(
        in.normal,
        in.instance_index
    );
#endif
#ifdef VERTEX_UVS_A
    if index == 0 {
        vertex_output.uv = surface.bottom_right_uv;
    } else if index == 1 {
        vertex_output.uv = surface.bottom_left_uv;
    } else if index == 2 {
        vertex_output.uv = surface.top_right_uv;
    } else if index == 3 {
        vertex_output.uv = surface.top_left_uv;
    }
#endif // VERTEX_UVS_A

    return vertex_output;
}

#ifdef MESH_PIPELINE
@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = gnd_default_material(in, is_front);

#ifdef BINDLESS
    let slot = mesh[in.instance_index].material_and_lightmap_bind_group_slot & 0xffffu;
    let texture = bindless_textures_2d[gnd_bindings[slot].texture];
    let texture_sampler = bindless_samplers_filtering[gnd_bindings[slot].texture_sampler];
#else // BINDLESS
    let texture = gnd_texture;
    let texture_sampler = gnd_sampler;
#endif // BINDLESS

    pbr_input.material.base_color = textureSample(texture, texture_sampler, in.uv);
    // Key out magenta
    if all(pbr_input.material.base_color.rgb == vec3(1.0, 0., 1.0)) {
        pbr_input.material.base_color.a = 0.;
    }

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}
#endif // MESH_PIPELINE

#ifdef PREPASS_PIPELINE
#ifdef PREPASS_FRAGMENT
@fragment
fn prepass_fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOut {
    var out: FragmentOutput;
#ifdef NORMAL_PREPASS
    out.normal = in.world_normal;
#endif
    return out;
}
#else // PREPASS_FRAGMENT
@fragment
fn prepass_fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) {
    return;
}
#endif // PREPASS_FRAGMENT
#endif // PREPASS_PIPELINE