#import bevy_pbr::{
    mesh_functions,
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_types::PbrInput,
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

#ifdef SPR_INDEXED_PIPELINE
@group(2) @binding(0) var spr_texture: texture_2d<u32>;
@group(2) @binding(1) var spr_palette: texture_1d<f32>;
#else ifdef SPR_TRUE_COLOR_PIPELINE
@group(2) @binding(0) var spr_texture: texture_2d<f32>;
@group(2) @binding(1) var spr_sampler: sampler;
#endif

fn spr_default_material(in: VertexOutput, is_front: bool) -> PbrInput {
    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);

    pbr_input.material.reflectance = 0.0;

    return pbr_input;
}

@vertex
fn vertex(in: Vertex, @builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var vertex_output: VertexOutput;

    var world_from_local = mesh_functions::get_world_from_local(in.instance_index);

    var dimensions = vec2<f32>(textureDimensions(spr_texture)) / 6.4;
    let position = vec4<f32>((in.position + vec3<f32>(0., 0.5, 0.)) * vec3<f32>(dimensions, 1.), 1.0);
    
#ifdef VERTEX_POSITIONS
    vertex_output.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, position);
    vertex_output.position = position_world_to_clip(vertex_output.world_position.xyz);
#endif

#ifdef VERTEX_NORMALS
    vertex_output.world_normal = mesh_functions::mesh_normal_local_to_world(
        in.normal,
        in.instance_index
    );
#endif

#ifdef VERTEX_UVS_A
    vertex_output.uv = in.uv;
#endif
#ifdef VERTEX_UVS_B
    vertex_output.uv_b = in.uv_b;
#endif


#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        world_from_local,
        in.tangent,
        in.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    vertex_output.color = in.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    vertex_output.instance_index = in.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    vertex_output.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        in.instance_index, world_from_local[3]);
#endif

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

    pbr_input.material.base_color = textureLoad(spr_palette, index, 0);
#else ifdef SPR_TRUE_COLOR_PIPELINE
    pbr_input.material.base_color = textureSample(spr_texture, spr_sampler, in.uv);
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