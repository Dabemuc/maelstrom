use graph::node::{Backend, Node};
use image::linear_image::LinearImage;

pub struct WhiteBalance {
    pub temp_val: f32,
    pub tint_val: f32,
}

impl Node for WhiteBalance {
    fn backend(&self) -> Backend {
        Backend::Cpu
    }

    fn process_cpu(&self, input: &LinearImage) -> LinearImage {
        let mut output = input.clone();

        let mut xy = kelvin_to_xy(self.temp_val);
        let mut uv = xy_to_uv(xy);

        // tint modifies the green-magenta axis
        uv[1] += self.tint_val * 0.001;
        uv[1] = uv[1].clamp(0.25, 0.40);

        xy = uv_to_xy(uv);

        let dst_xyz = xy_to_xyz(xy);

        let src_xyz = [0.95047, 1.0, 1.08883]; // D65

        let m = bradford_adaptation(src_xyz, dst_xyz);

        for px in output.data.chunks_exact_mut(3) {
            let r = px[0];
            let g = px[1];
            let b = px[2];

            px[0] = m[0][0] * r + m[0][1] * g + m[0][2] * b;
            px[1] = m[1][0] * r + m[1][1] * g + m[1][2] * b;
            px[2] = m[2][0] * r + m[2][1] * g + m[2][2] * b;
        }

        output
    }
}

fn kelvin_to_xy(t: f32) -> [f32; 2] {
    let x;

    if (1667.0..=4000.0).contains(&t) {
        x = -0.2661239e9 / (t * t * t) - 0.2343580e6 / (t * t) + 0.8776956e3 / t + 0.179910;
    } else {
        x = -3.0258469e9 / (t * t * t) + 2.1070379e6 / (t * t) + 0.2226347e3 / t + 0.240390;
    }

    let y = -3.0 * x * x + 2.87 * x - 0.275;

    [x as f32, y as f32]
}

fn xy_to_uv(xy: [f32; 2]) -> [f32; 2] {
    let x = xy[0];
    let y = xy[1];

    let d = -2.0 * x + 12.0 * y + 3.0;

    let u = (4.0 * x) / d;
    let v = (6.0 * y) / d;

    [u, v]
}

fn uv_to_xy(uv: [f32; 2]) -> [f32; 2] {
    let u = uv[0];
    let v = uv[1];

    let x = (9.0 * u) / (6.0 * u - 16.0 * v + 12.0);
    let y = (4.0 * v) / (6.0 * u - 16.0 * v + 12.0);

    [x, y]
}

fn xy_to_xyz(xy: [f32; 2]) -> [f32; 3] {
    let x = xy[0];
    let y = xy[1];

    let x_o = x / y;
    let y_o = 1.0;
    let z_o = (1.0 - x - y) / y;

    [x_o, y_o, z_o]
}

fn bradford_adaptation(src_xyz: [f32; 3], dst_xyz: [f32; 3]) -> [[f32; 3]; 3] {
    let mb = [
        [0.8951, 0.2664, -0.1614],
        [-0.7502, 1.7135, 0.0367],
        [0.0389, -0.0685, 1.0296],
    ];

    let mb_inv = [
        [0.9869929, -0.1470543, 0.1599627],
        [0.4323053, 0.5183603, 0.0492912],
        [-0.0085287, 0.0400428, 0.9684867],
    ];

    let src_lms = mul_vec3(mb, src_xyz);
    let dst_lms = mul_vec3(mb, dst_xyz);

    let scale = [
        dst_lms[0] / src_lms[0],
        dst_lms[1] / src_lms[1],
        dst_lms[2] / src_lms[2],
    ];

    let diag = [
        [scale[0], 0.0, 0.0],
        [0.0, scale[1], 0.0],
        [0.0, 0.0, scale[2]],
    ];

    mul_mat3(mb_inv, mul_mat3(diag, mb))
}

fn mul_vec3(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn mul_mat3(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut r = [[0.0; 3]; 3];

    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }

    r
}
