#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
}

@group(2) @binding(0) var rsm_texture: texture_2d<f32>;
@group(2) @binding(1) var rsm_sampler: sampler;

#ifdef MESH_PIPELINE
fn rsm_default_material(in: VertexOutput, is_front: bool) -> PbrInput {
    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);

    pbr_input.material.reflectance = 0.0;
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_MASK;

    return pbr_input;
}
#endif

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var color = textureSample(rsm_texture, rsm_sampler, in.uv);
    if all(color.rgb == vec3(1.0, 0., 1.0)) {
        discard;
    }

    var out: FragmentOutput;
#ifdef DEPTH_CLAMP_ORTHO
    out.frag_depth = in.clip_position_unclamped.z;
#endif
    return out;
}
