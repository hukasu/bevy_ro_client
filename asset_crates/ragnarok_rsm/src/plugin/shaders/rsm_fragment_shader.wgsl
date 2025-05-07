#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::{
        PbrInput,
        STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND,
    },
}

@group(2) @binding(0) var rsm_texture: texture_2d<f32>;
@group(2) @binding(1) var rsm_sampler: sampler;

fn rsm_default_material(in: VertexOutput, is_front: bool) -> PbrInput {
    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);

    pbr_input.material.reflectance = vec3(0.0);
    pbr_input.material.flags = STANDARD_MATERIAL_FLAGS_ALPHA_MODE_BLEND;

    return pbr_input;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var color = textureSample(rsm_texture, rsm_sampler, in.uv);
    if all(color.rgb == vec3(1.0, 0., 1.0)) {
        discard;
    }

    var pbr_input = rsm_default_material(in, is_front);
    pbr_input.material.base_color = color;

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}
