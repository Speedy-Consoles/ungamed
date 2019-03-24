use std::f32::consts::PI;

use cgmath::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Color { r, g, b }
    }

    pub fn from_hsl(mut h: f32, s: f32, l: f32) -> Self {
        h = ((h % (2.0 * PI)) + (2.0 * PI)) % (2.0 * PI);
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / PI * 3.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;
        let cm = c + m;
        let xm = x + m;
        match (h / PI * 3.0).floor() as i64 {
            0 => Color { r: cm, g: xm, b:  m },
            1 => Color { r: xm, g: cm, b:  m },
            2 => Color { r:  m, g: cm, b: xm },
            3 => Color { r:  m, g: xm, b: cm },
            4 => Color { r: xm, g:  m, b: cm },
            _ => Color { r: cm, g:  m, b: xm },
        }
    }

    pub fn from_hsv(mut h: f32, s: f32, v: f32) -> Self {
        h = ((h % (2.0 * PI)) + (2.0 * PI)) % (2.0 * PI);
        let c = v * s;
        let x = c * (1.0 - ((h / PI * 3.0) % 2.0 - 1.0).abs());
        let m = v - c;
        let cm = c + m;
        let xm = x + m;
        match (h / PI * 3.0).floor() as i64 {
            0 => Color { r: cm, g: xm, b:  m },
            1 => Color { r: xm, g: cm, b:  m },
            2 => Color { r:  m, g: cm, b: xm },
            3 => Color { r:  m, g: xm, b: cm },
            4 => Color { r: xm, g:  m, b: cm },
            _ => Color { r: cm, g:  m, b: xm },
        }
    }

    pub const fn white()   -> Color { Color { r: 1.0, g: 1.0, b: 1.0 } }
    pub const fn black()   -> Color { Color { r: 0.0, g: 0.0, b: 0.0 } }
    pub const fn red()     -> Color { Color { r: 1.0, g: 0.0, b: 0.0 } }
    pub const fn green()   -> Color { Color { r: 0.0, g: 1.0, b: 0.0 } }
    pub const fn blue()    -> Color { Color { r: 0.0, g: 0.0, b: 1.0 } }
    pub const fn cyan()    -> Color { Color { r: 0.0, g: 1.0, b: 1.0 } }
    pub const fn magenta() -> Color { Color { r: 1.0, g: 0.0, b: 1.0 } }
    pub const fn yellow()  -> Color { Color { r: 1.0, g: 1.0, b: 0.0 } }
}

impl From<[f32; 3]> for Color {
    fn from(a: [f32; 3]) -> Self {
        Color { r: a[0], g: a[1], b: a[2] }
    }
}

impl From<Vector3<f32>> for Color {
    fn from(v: Vector3<f32>) -> Self {
        Color { r: v.x, g: v.y, b: v.z }
    }
}

impl Into<[f32; 3]> for Color {
    fn into(self) -> [f32; 3] {
        [self.r, self.g, self.b]
    }
}

impl Into<Vector3<f32>> for Color {
    fn into(self) -> Vector3<f32> {
        Vector3::new(self.r, self.g, self.b)
    }
}