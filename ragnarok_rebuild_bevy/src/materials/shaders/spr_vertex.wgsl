#import bevy_pbr::{
    mesh_functions,
    forward_io::{Vertex, VertexOutput},
    view_transformations::position_world_to_clip,
}

#ifdef SPR_INDEXED_PIPELINE
@group(2) @binding(0) var texture: texture_2d<u32>;
#else ifdef SPR_TRUE_COLOR_PIPELINE
@group(2) @binding(0) var texture: texture_2d<f32>;
#endif

@vertex
fn vertex(in: Vertex, @builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var vertex_output: VertexOutput;

    var world_from_local = mesh_functions::get_world_from_local(in.instance_index);

    var dimensions = vec2<f32>(textureDimensions(texture) / 5);
    var offset: vec3<f32>;
    if vertex_index == 0 {
        // Top left
        offset = vec3<f32>(-dimensions.x / 2., dimensions.y, 0.);
    } else if vertex_index == 1 {
        // Top right
        offset = vec3<f32>(dimensions.x / 2., dimensions.y, 0.);
    } else if vertex_index == 2 {
        // Bottom left
        offset = vec3<f32>(-dimensions.x / 2., 0., 0.);
    } else if vertex_index == 3 {
        // Bottom right
        offset = vec3<f32>(dimensions.x / 2., 0., 0.);
    }
    let position = vec4<f32>(offset, 1.0);
    
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