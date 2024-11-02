#import bevy_pbr::{
    mesh_functions,
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_functions::alpha_discard,
    pbr_types::{PbrInput, STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND},
    view_transformations::position_world_to_clip,
    mesh_view_bindings::globals,
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

struct Wave {
    wave_height: f32,
    wave_speed: f32,
    wave_pitch: f32,
}

@group(2) @binding(0) var water_texture: texture_2d<f32>;
@group(2) @binding(1) var water_sample: sampler;
@group(2) @binding(2) var<uniform> wave: Wave;

fn water_plane_default_material(in: VertexOutput, is_front: bool) -> PbrInput {
    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);

    pbr_input.material.reflectance = 0.0;
#ifndef OPAQUE_WATER_PLANE
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND;
#endif

    return pbr_input;
}

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var vertex_output: VertexOutput;
    
    var world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    var normalized_in = world_from_local * vec4(
        in.position.x,
        in.position.y,
        in.position.z,
        1.,
    ) / 2.;
    var position = vec4(
        in.position + vec3(
            0.,
            wave.wave_height * sin((normalized_in.x - normalized_in.z) * wave.wave_pitch + globals.time * wave.wave_speed),
            0.),
        1.);
    vertex_output.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, position);
    vertex_output.position = position_world_to_clip(vertex_output.world_position.xyz);
    
    vertex_output.world_normal = mesh_functions::mesh_normal_local_to_world(
        in.normal,
        in.instance_index
    );
    vertex_output.uv = in.uv;

    return vertex_output;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = water_plane_default_material(in, is_front);

    var scaled_uv = in.uv * (64. / vec2<f32>(textureDimensions(water_texture)));
    pbr_input.material.base_color = textureSample(water_texture, water_sample, scaled_uv);
#ifndef OPAQUE_WATER_PLANE
    pbr_input.material.base_color.a = 0.5;
#endif

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