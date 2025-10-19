#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
    mesh_view_bindings::globals,
}
#ifdef MESH_PIPELINE
#import bevy_pbr::{
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_types::{PbrInput, STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND},
}
#endif

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{Vertex, VertexOutput},
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

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var water_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var water_sample: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> wave: Wave;

#ifdef MESH_PIPELINE
fn water_plane_default_material(in: VertexOutput, is_front: bool) -> PbrInput {
    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);

    pbr_input.material.reflectance = vec3(0.0);
#ifndef OPAQUE_WATER_PLANE
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND;
#endif

    return pbr_input;
}
#endif

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var vertex_output: VertexOutput;
    
    var world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    var normalized_in = world_from_local * vec4(in.position, 1.) / 2.;

    let param = (normalized_in.x - normalized_in.z) * wave.wave_pitch + globals.time * wave.wave_speed;
    let y_offset = wave.wave_height * sin(param);

    var position = vec4(in.position + vec3(0., y_offset, 0.), 1.);
    vertex_output.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, position);
    vertex_output.position = position_world_to_clip(vertex_output.world_position.xyz);

    let derivate = wave.wave_height * wave.wave_speed * cos(param);
    let slope_angle = atan2(1, -derivate);

#ifdef VERTEX_NORMALS || NORMAL_PREPASS_OR_DEFERRED_PREPASS
    vertex_output.world_normal = mesh_functions::mesh_normal_local_to_world(
        vec3(-pow(2., -0.5) * cos(slope_angle), -sin(slope_angle), -pow(2., -0.5) * cos(slope_angle)),
        in.instance_index
    );
#endif
#ifdef VERTEX_UV_A
    vertex_output.uv = in.uv;
#endif

    return vertex_output;
}

#ifdef MESH_PIPELINE
@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = water_plane_default_material(in, is_front);

    var scaled_uv = in.uv * (64. / vec2<f32>(textureDimensions(water_texture)));
    pbr_input.material.base_color = textureSample(water_texture, water_sample, scaled_uv);
#ifndef OPAQUE_WATER_PLANE
    pbr_input.material.base_color.a = 144. / 255.;
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
#endif // MESH_PIPELINE