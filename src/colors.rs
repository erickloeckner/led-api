use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ColorRgb {
    r: u8,
    g: u8,
    b: u8,
}

impl ColorRgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r,
            g: g,
            b: b,
        }
    }

    pub fn set_rgb(&mut self, rgb: &ColorRgb) {
        self.r = rgb.r;
        self.g = rgb.g;
        self.b = rgb.b;
    }

    pub fn set_r(&mut self, r: u8) {
        self.r = r;
    }

    pub fn get_r(&self) -> u8 {
        self.r
    }

    pub fn set_g(&mut self, g: u8) {
        self.g = g;
    }

    pub fn get_g(&self) -> u8 {
        self.g
    }

    pub fn set_b(&mut self, b: u8) {
        self.b = b;
    }

    pub fn get_b(&self) -> u8 {
        self.b
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct ColorHsv {
    h: f32,
    s: f32,
    v: f32,
}

impl ColorHsv {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        Self { 
            h: h.max(0.0).min(1.0), 
            s: s.max(0.0).min(1.0), 
            v: v.max(0.0).min(1.0),
        }
    }

    pub fn from_le_bytes(bytes: [u8; 12]) -> Self {
        let mut h = [0; 4];
        h.iter_mut().zip(bytes.iter().take(4)).for_each(|(i, j)| *i = *j );
        let mut s = [0; 4];
        s.iter_mut().zip(bytes.iter().skip(4).take(4)).for_each(|(i, j)| *i = *j );
        let mut v = [0; 4];
        v.iter_mut().zip(bytes.iter().skip(8).take(4)).for_each(|(i, j)| *i = *j );

        Self::new(f32::from_le_bytes(h), f32::from_le_bytes(s), f32::from_le_bytes(v))
    }

    pub fn to_le_bytes(&self) -> [u8; 12] {
        let mut out = [0; 12];
        let h = self.h.to_le_bytes();
        let s = self.s.to_le_bytes();
        let v = self.v.to_le_bytes();
        out.iter_mut().take(4).zip(h).for_each(|(v, c)| *v = c );
        out.iter_mut().skip(4).take(4).zip(s).for_each(|(v, c)| *v = c );
        out.iter_mut().skip(8).take(4).zip(v).for_each(|(v, c)| *v = c );
        out
    }

    pub fn set_hsv(&mut self, hsv: &ColorHsv) {
        self.h = hsv.h.max(0.0).min(1.0);
        self.s = hsv.s.max(0.0).min(1.0);
        self.v = hsv.v.max(0.0).min(1.0);
    }

    pub fn set_h(&mut self, h: f32) {
        self.h = h.max(0.0).min(1.0);
    }

    pub fn get_h(&self) -> f32 {
        self.h
    }

    pub fn set_s(&mut self, s: f32) {
        self.s = s.max(0.0).min(1.0);
    }

    pub fn get_s(&self) -> f32 {
        self.s
    }

    pub fn set_v(&mut self, v: f32) {
        self.v = v.max(0.0).min(1.0);
    }

    pub fn get_v(&self) -> f32 {
        self.v
    }

    pub fn to_rgb(&self) -> ColorRgb {
        let mut out = ColorRgb { r: 0, g: 0, b: 0 };
        let h_decimal = (self.h * 6.0) - (((self.h * 6.0) as u8) as f32);
        match (self.h * 6.0 % 6.0) as u8 {
            0 => {
                out.r = (self.v * 255.0) as u8;
                out.g = ((self.v * (1.0 - self.s * (1.0 - h_decimal))) * 255.0) as u8;
                out.b = ((self.v * (1.0 - self.s)) * 255.0) as u8;
            }
            1 => {
                out.r = ((self.v * (1.0 - self.s * h_decimal)) * 255.0) as u8;
                out.g = (self.v * 255.0) as u8;
                out.b = ((self.v * (1.0 - self.s)) * 255.0) as u8;
            }
            2 => {
                out.r = ((self.v * (1.0 - self.s)) * 255.0) as u8;
                out.g = (self.v * 255.0) as u8;
                out.b = ((self.v * (1.0 - self.s * (1.0 - h_decimal))) * 255.0) as u8;
            }
            3 => {
                out.r = ((self.v * (1.0 - self.s)) * 255.0) as u8;
                out.g = ((self.v * (1.0 - self.s * h_decimal)) * 255.0) as u8;
                out.b = (self.v * 255.0) as u8;
            }
            4 => {
                out.r = ((self.v * (1.0 - self.s * (1.0 - h_decimal))) * 255.0) as u8;
                out.g = ((self.v * (1.0 - self.s)) * 255.0) as u8;
                out.b = (self.v * 255.0) as u8;
            }
            5 => {
                out.r = (self.v * 255.0) as u8;
                out.g = ((self.v * (1.0 - self.s)) * 255.0) as u8;
                out.b = ((self.v * (1.0 - self.s * h_decimal)) * 255.0) as u8;
            }
            _ => (),
        }
        out
    }
}

pub fn hsv_interp(col1: &ColorHsv, col2: &ColorHsv, pos: f32) -> ColorHsv {
    let h_range = col1.h - col2.h;
    let s_range = col1.s - col2.s;
    let v_range = col1.v - col2.v;
    let pos_clip = pos.max(0.0).min(1.0);

    let h_out = col1.h - (h_range * pos_clip);
    let s_out = col1.s - (s_range * pos_clip);
    let v_out = col1.v - (v_range * pos_clip);

    ColorHsv { h: h_out, s: s_out, v: v_out }
}

pub fn hsv_interp_3(col1: &ColorHsv, col2: &ColorHsv, col3: &ColorHsv, pos: f32) -> ColorHsv {
	let mut pos_clip = 0.0;
    if pos < 0.0 {
        pos_clip = pos.max(-1.0);
    } else if pos > 0.0 {
        pos_clip = pos.min(1.0);
    } else {
        pos_clip = 0.0;
    }
	if pos_clip > 0.0 {
        let h_range = col1.h - col2.h;
        let s_range = col1.s - col2.s;
        let v_range = col1.v - col2.v;
        
        let h_out = col1.h - (h_range * pos_clip);
        let s_out = col1.s - (s_range * pos_clip);
        let v_out = col1.v - (v_range * pos_clip);

        ColorHsv { h: h_out, s: s_out, v: v_out }
    } else if pos_clip < 0.0 {
        let h_range = col1.h - col3.h;
        let s_range = col1.s - col3.s;
        let v_range = col1.v - col3.v;
        
        let h_out = col1.h - (h_range * (pos_clip * -1.0));
        let s_out = col1.s - (s_range * (pos_clip * -1.0));
        let v_out = col1.v - (v_range * (pos_clip * -1.0));

        ColorHsv { h: h_out, s: s_out, v: v_out }
    } else {
        let h_out = col1.h;
        let s_out = col1.s;
        let v_out = col1.v;

        ColorHsv { h: h_out, s: s_out, v: v_out }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colorhsv() {
        // -test ColorHsv::new() with pure black
        assert_eq!(ColorHsv::new(0.0, 0.0, 0.0), ColorHsv {h: 0.0, s: 0.0, v: 0.0});
        let col_hsv_white = ColorHsv::new(0.0, 0.0, 1.0);
        let col_rgb_white = ColorRgb::new(255, 255, 255);
        let col_hsv_red = ColorHsv::new(0.0, 1.0, 1.0);
        let col_rgb_red = ColorRgb::new(255, 0, 0);
        // array equal to (0.0, 0.0, 1.0) converted to little endian
        let byte_array_1 = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128, 63];
        
        // -test- converting from HSV to bytes and bytes to HSV
        assert_eq!(col_hsv_white.to_le_bytes(), byte_array_1);
        assert_eq!(ColorHsv::from_le_bytes(byte_array_1), col_hsv_white);

        // -test- converting HSV to RGB
        assert_eq!(col_hsv_white.to_rgb(), col_rgb_white);
        assert_eq!(col_hsv_red.to_rgb(), col_rgb_red);
    }

    #[test]
    fn test_interp_functions() {
        let col_hsv_white = ColorHsv::new(0.0, 0.0, 1.0);
        let col_hsv_black = ColorHsv::new(0.0, 0.0, 0.0);
        let col_hsv_h0 = ColorHsv::new(0.0, 1.0, 1.0);
        let col_hsv_h25 = ColorHsv::new(0.25, 1.0, 1.0);
        let col_hsv_h50 = ColorHsv::new(0.5, 1.0, 1.0);
        let col_hsv_h75 = ColorHsv::new(0.75, 1.0, 1.0);
        let col_hsv_h100 = ColorHsv::new(1.0, 1.0, 1.0);

        // -test- HSV interpolation functions
        assert_eq!(hsv_interp(&col_hsv_black, &col_hsv_white, 0.0), col_hsv_black);
        assert_eq!(hsv_interp(&col_hsv_black, &col_hsv_white, 0.5), ColorHsv::new(0.0, 0.0, 0.5));
        assert_eq!(hsv_interp(&col_hsv_black, &col_hsv_white, 1.0), col_hsv_white);

        assert_eq!(hsv_interp_3(&col_hsv_h50, &col_hsv_h100, &col_hsv_h0, 0.0), col_hsv_h50);
        assert_eq!(hsv_interp_3(&col_hsv_h50, &col_hsv_h100, &col_hsv_h0, 0.5), col_hsv_h75);
        assert_eq!(hsv_interp_3(&col_hsv_h50, &col_hsv_h100, &col_hsv_h0, 1.0), col_hsv_h100);
        assert_eq!(hsv_interp_3(&col_hsv_h50, &col_hsv_h100, &col_hsv_h0, -0.5), col_hsv_h25);
        assert_eq!(hsv_interp_3(&col_hsv_h50, &col_hsv_h100, &col_hsv_h0, -1.0), col_hsv_h0);
    }
}
