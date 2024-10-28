#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
}

@group(2) @binding(0) var rsm_texture: texture_2d<f32>;
@group(2) @binding(1) var rsm_sampler: sampler;

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
