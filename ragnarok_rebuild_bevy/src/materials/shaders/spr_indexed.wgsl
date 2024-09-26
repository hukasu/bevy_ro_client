#import bevy_pbr::{
    pbr_fragment::pbr_input_from_vertex_output,
    pbr_types::PbrInput
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

@group(2) @binding(0) var index_texture: texture_2d<u32>;
@group(2) @binding(1) var palette_texture: texture_1d<f32>;

fn spr_indexed_default_material(in: VertexOutput, is_front: bool) -> PbrInput {
    var pbr_input = pbr_input_from_vertex_output(in, is_front, false);

    pbr_input.material.reflectance = 0.0;

    return pbr_input;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = spr_indexed_default_material(in, is_front);

    let index_texture_dimensions = textureDimensions(index_texture);
    let index_texture_coords = vec2<u32>(vec2<f32>(index_texture_dimensions) * in.uv);
    let index = textureLoad(index_texture, index_texture_coords, 0).x;

    pbr_input.material.base_color = textureLoad(palette_texture, index, 0);

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