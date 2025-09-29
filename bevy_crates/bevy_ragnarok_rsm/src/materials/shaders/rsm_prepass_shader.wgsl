#import bevy_pbr::prepass_io::VertexOutput
#ifdef PREPASS_FRAGMENT
#import bevy_pbr::prepass_io::FragmentOutput
#endif

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var rsm_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var rsm_sampler: sampler;

@fragment
fn fragment(
    in: VertexOutput,
#ifdef PREPASS_FRAGMENT
) -> FragmentOutput {
#else 
) {
#endif
    var color = textureSample(rsm_texture, rsm_sampler, in.uv);
    if all(color.rgb == vec3(1.0, 0., 1.0)) {
        discard;
    }

#ifdef PREPASS_FRAGMENT
    var out: FragmentOutput;
#ifdef DEPTH_CLAMP_ORTHO
    out.frag_depth = in.clip_position_unclamped.z;
#endif
    return out;
#endif
}