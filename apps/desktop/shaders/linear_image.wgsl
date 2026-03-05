struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct GammaSettings {
    apply_srgb: u32,
};

@group(0) @binding(0)
var tex: texture_2d<f32>;

@group(0) @binding(1)
var<uniform> settings: GammaSettings;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(input.position, 0.0, 1.0);
    out.uv = input.uv;
    return out;
}

fn linear_to_srgb(v: f32) -> f32 {
    if (v <= 0.0031308) {
        return 12.92 * v;
    }
    return 1.055 * pow(v, 1.0 / 2.4) - 0.055;
}

fn maybe_convert(color: vec3<f32>) -> vec3<f32> {
    if (settings.apply_srgb == 1u) {
        return vec3<f32>(
            linear_to_srgb(color.r),
            linear_to_srgb(color.g),
            linear_to_srgb(color.b)
        );
    }

    return color;
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let dims = textureDimensions(tex);
    let coord_f = vec2<f32>(dims) * uv;
    let coord = vec2<i32>(min(coord_f, vec2<f32>(dims) - vec2<f32>(1.0)));
    let c = textureLoad(tex, coord, 0);
    let rgb = maybe_convert(c.rgb);
    return vec4<f32>(rgb, c.a);
}
