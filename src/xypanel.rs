use minvect::*;
use crate::put_rect;
use glow_mesh::{xyzrgba::*};


pub struct XYPanel {
    pub transform: [f32; 9],
    pub p: Vec2,
}

fn inverse(mat: &[f32; 9]) -> [f32; 9] {
    let a = mat[0];
    let b = mat[1];
    let c = mat[2];
    let d = mat[3];
    let e = mat[4];
    let f = mat[5];
    let g = mat[6];
    let h = mat[7];
    let i = mat[8];

    let det = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
    
    if det == 0.0 {
        // The matrix is singular, return an identity matrix as a placeholder
        return [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    }

    let inv_det = 1.0 / det;

    let result = [
        (e * i - f * h) * inv_det,
        (c * h - b * i) * inv_det,
        (b * f - c * e) * inv_det,
        (f * g - d * i) * inv_det,
        (a * i - c * g) * inv_det,
        (c * d - a * f) * inv_det,
        (d * h - e * g) * inv_det,
        (b * g - a * h) * inv_det,
        (a * e - b * d) * inv_det,
    ];

    result
}


fn trans(p: Vec2, m: &[f32; 9]) -> Vec2 {
    let x = p.x * m[0] + p.y * m[1] + m[2];
    let y = p.x * m[3] + p.y * m[4] + m[5];
    vec2(x, y)    
}

impl XYPanel {
    pub fn new(transform: [f32; 9], initial_p: Vec2) -> Self {
        // let transform = [
        //     2.0 / (max.x - min.x), 0.0, -(max.x + min.x) / (max.x - min.x),
        //     0.0, 2.0 / (max.y - min.y), -(max.y + min.y) / (max.y - min.y),
        //     0.0, 0.0, 1.0,
        // ];
        XYPanel {
            transform,
            p: initial_p,
        }
    }
    // pub fn new(min: Vec2, max: Vec2, initial_p: Vec2) -> Self {
    //     let transform = [
    //         2.0 / (max.x - min.x), 0.0, -(max.x + min.x) / (max.x - min.x),
    //         0.0, 2.0 / (max.y - min.y), -(max.y + min.y) / (max.y - min.y),
    //         0.0, 0.0, 1.0,
    //     ];
    //     XYPanel {
    //         transform,
    //         p: initial_p,
    //     }
    // }

    pub fn update(&mut self, p: Vec2) -> bool {
        let pt = trans(p, &inverse(&self.transform));
        // if within unit square update this p
        if pt.x >= -1.0 && pt.x <= 1.0 && pt.y >= -1.0 && pt.y <= 1.0 {
            self.p = pt;
            true
        } else {
            false
        }
    }

    pub fn push_geometry(&self, buf: &mut Vec<XYZRGBA>, depth: f32) {
        let col_panel = vec4(0.0, 0.7, 0.3, 1.0).hsv_to_rgb();
        let col_lines = vec4(0.0, 0.7, 0.6, 1.0).hsv_to_rgb();

        // and push the rect of this which would i guess be ndc transformed by transform
        let p1 = vec2(-1.0, -1.0);
        let p2 = vec2(1.0, 1.0);
        let t = self.transform;
        let p1t = trans(p1, &t);
        let p2t = trans(p2, &t);
        put_rect(buf, p1t, p2t, col_panel, depth);
        
        let p1 = vec2(-1.0, 0.0);
        let p2 = vec2(1.0, 0.0);
        let p1t = trans(p1, &t);
        let p2t = trans(p2, &t);
        put_line(buf, p1t, p2t, 0.01, col_lines, depth - 0.01);

        let p1 = vec2(0.0, -1.0);
        let p2 = vec2(0.0, 1.0);
        let p1t = trans(p1, &t);
        let p2t = trans(p2, &t);
        put_line(buf, p1t, p2t, 0.01, col_lines, depth - 0.01);

        put_crosshair(buf, trans(self.p, &t));
    }
}
    
// needs to put crosshair at p brah or inverse of its p


fn put_crosshair(buf: &mut Vec<XYZRGBA>, p: Vec2) {
    let d = -0.5;
    let c = vec4(1.0, 1.0, 0.0, 1.0);
    let w = 0.02;
    let h = 0.06;
    
    let wx = vec2(w, 0.0);
    let wy = vec2(0.0, w);
    
    let hx = vec2(h, 0.0);
    let hy = vec2(0.0, h);

    put_rect(buf, p - wx - wy, p + wx - wy - hy, c, d);
    put_rect(buf, p - wx + wy, p + wx + wy + hy, c, d);
    put_rect(buf, p - wx - wy - hx, p - wx + wy, c, d);
    put_rect(buf, p + wx - wy, p + wx + wy + hx, c, d);
}