use crate::colors::{hsv_interp, ColorRgb, ColorHsv};
use crate::sprites::Sprite;

#[derive (Clone)]
pub enum LedType {
    Apa102,
    Ws2801,
}

pub struct Leds {
    led_type: LedType,
    len: usize,
    //~ buffer: [u8; B],
    buffer: Vec<u8>,
}

impl Leds {
    pub fn new(len: usize, led_type: LedType) -> Self {
        let mut buffer_size: usize = 0;
        match led_type {
            LedType::Apa102 => { buffer_size = 4 + (len * 4) + ((len + 1) / 2); }
            LedType::Ws2801 => { buffer_size = len * 3; }
        }
        let mut buffer = Vec::with_capacity(buffer_size);
        for _i in 0..buffer_size {
            buffer.push(0);
        }
        buffer.iter_mut().for_each(|v| *v = 0 );
        Self {
            led_type: led_type,
            len: len,
            //~ buffer: [0; B],
            buffer: buffer,
        }
    }

    pub fn all_off(&mut self) {
        match self.led_type {
            LedType::Apa102 => {
                let len = self.len;
                self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
                    if i < len {
                        v[0] = 255;
                        v[1] = 0;
                        v[2] = 0;
                        v[3] = 0;
                    }
                });
            }
            LedType::Ws2801 => {
                self.buffer.iter_mut().for_each(|v| *v = 0 );
            }
        }
    }
    
    pub fn get_buffer(&self) -> &[u8] {
        &self.buffer
    }
    
    pub fn set_led(&mut self, color: ColorRgb, index: usize) {
        if index < self.len {
            match self.led_type {
                LedType::Apa102 => {
                    match self.buffer.chunks_mut(4).skip(1).nth(index) {
                        Some(v) => {
                            v[0] = 255;
                            v[1] = color.get_b();
                            v[2] = color.get_g();
                            v[3] = color.get_r();
                        },
                        None => {},
                    }
                }
                LedType::Ws2801 => {
                    match self.buffer.chunks_mut(3).nth(index) {
                        Some(v) => {
                            v[0] = color.get_r();
                            v[1] = color.get_g();
                            v[2] = color.get_b();
                        },
                        None => {},
                    }
                }
            }
        }
    }

    pub fn fill_gradient(&mut self, start: &ColorHsv, end: &ColorHsv) {
        match self.led_type {
            LedType::Apa102 => {
                let len = self.len;
                self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
                    if i < len {
                        let pos = (i as f32) / ((len - 1) as f32);
                        let rgb = hsv_interp(&start, &end, pos).to_rgb();
                        v[0] = 255;
                        v[1] = rgb.get_b();
                        v[2] = rgb.get_g();
                        v[3] = rgb.get_r();
                    }
                });
            }
            LedType::Ws2801 => {
                self.buffer.chunks_mut(3).enumerate().for_each(|(i, v)| {
                    let pos = (i as f32) / ((self.len - 1) as f32);
                    let rgb = hsv_interp(&start, &end, pos).to_rgb();
                    v[0] = rgb.get_r();
                    v[1] = rgb.get_g();
                    v[2] = rgb.get_b();
                });
            }
        }
    }

    pub fn fill_gradient_dual(&mut self, start: &ColorHsv, end: &ColorHsv) {
        match self.led_type {
            LedType::Apa102 => {
                self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
                    if i < self.len {
                        let pos = (i as f32) / ((self.len - 1) as f32);
                        let mut pos_bipolar = pos * 2.0 - 1.0;
                        if pos_bipolar < 0.0 { pos_bipolar = pos_bipolar * -1.0 }
                        let rgb = hsv_interp(&end, &start, pos_bipolar).to_rgb();
                        v[0] = 255;
                        v[1] = rgb.get_b();
                        v[2] = rgb.get_g();
                        v[3] = rgb.get_r();
                    }
                });
            }
            LedType::Ws2801 => {                
                self.buffer.chunks_mut(3).enumerate().for_each(|(i, v)| {
                    let pos = (i as f32) / ((self.len - 1) as f32);
                    let mut pos_bipolar = pos * 2.0 - 1.0;
                    if pos_bipolar < 0.0 { pos_bipolar = pos_bipolar * -1.0 }
                    let rgb = hsv_interp(&end, &start, pos_bipolar).to_rgb();
                    v[0] = rgb.get_r();
                    v[1] = rgb.get_g();
                    v[2] = rgb.get_b();
                });
            }
        }
    }
    
    pub fn fill_gradient_triple(&mut self, col1: &ColorHsv, col2: &ColorHsv, col3: &ColorHsv) {
        match self.led_type {
            LedType::Apa102 => {
                self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
                    if i < self.len {
                        let pos = (i as f32) / ((self.len - 1) as f32);
                        let mut pos_bipolar = pos * 2.0 - 1.0;
                        let mut rgb = ColorRgb::new(0, 0, 0);
                        if pos_bipolar < 0.0 {
                            pos_bipolar = pos_bipolar * -1.0;
                            rgb = hsv_interp(&col2, &col1, pos_bipolar).to_rgb();
                        } else {
                            rgb = hsv_interp(&col2, &col3, pos).to_rgb();
                        }
                        //~ let rgb = hsv_interp(&end, &start, pos_bipolar).to_rgb();
                        v[0] = 255;
                        v[1] = rgb.get_b();
                        v[2] = rgb.get_g();
                        v[3] = rgb.get_r();
                    }
                });
            }
            LedType::Ws2801 => {                
                self.buffer.chunks_mut(3).enumerate().for_each(|(i, v)| {
                    let pos = (i as f32) / ((self.len - 1) as f32);
                    let mut pos_bipolar = pos * 2.0 - 1.0;
                    let mut rgb = ColorRgb::new(0, 0, 0);
                    if pos_bipolar < 0.0 {
                        pos_bipolar = pos_bipolar * -1.0;
                        rgb = hsv_interp(&col2, &col1, pos_bipolar).to_rgb();
                    } else {
                        rgb = hsv_interp(&col2, &col3, pos).to_rgb();
                    }
                    v[0] = rgb.get_r();
                    v[1] = rgb.get_g();
                    v[2] = rgb.get_b();
                });
            }
        }
    }
    
    pub fn fill_sine(&mut self, col1: &ColorHsv, col2: &ColorHsv, col3: &ColorHsv, phase: f32) {
        match self.led_type {
            LedType::Apa102 => {
                self.buffer.chunks_mut(4).skip(1).enumerate().for_each(|(i, v)| {
                    if i < self.len {
                        let pos = (i as f32) / ((self.len - 1) as f32);
                        let pos_triangle = (((pos + phase) % 1.0) * 2.0 - 1.0).abs() * 2.0 - 1.0;
                        let mut rgb = ColorRgb::new(0, 0, 0);
                        if pos_triangle < 0.0 {
                            rgb = hsv_interp(&col1, &col2, pos_triangle * -1.0).to_rgb();
                        } else {
                            rgb = hsv_interp(&col1, &col3, pos_triangle).to_rgb();
                        }
                        v[0] = 255;
                        v[1] = rgb.get_b();
                        v[2] = rgb.get_g();
                        v[3] = rgb.get_r();
                    }
                });
            }
            LedType::Ws2801 => {
                self.buffer.chunks_mut(3).enumerate().for_each(|(i, v)| {
                    let pos = (i as f32) / ((self.len - 1) as f32);
                    let pos_triangle = (((pos + phase) % 1.0) * 2.0 - 1.0).abs() * 2.0 - 1.0;
                    let mut rgb = ColorRgb::new(0, 0, 0);
                    if pos_triangle < 0.0 {
                        rgb = hsv_interp(&col1, &col2, pos_triangle * -1.0).to_rgb();
                    } else {
                        rgb = hsv_interp(&col1, &col3, pos_triangle).to_rgb();
                    }
                    v[0] = rgb.get_r();
                    v[1] = rgb.get_g();
                    v[2] = rgb.get_b();
                });
            }
        }
    }

    pub fn fill_sprites(&mut self, col1: &ColorHsv, col2: &ColorHsv, col3: &ColorHsv, sprites: &Vec<Sprite>) {
        let mut sprite_values = Vec::with_capacity(self.len);
        for _ in 0..self.len { sprite_values.push(0.0) }
        //for sprite in sprites {
        for (i, v) in sprite_values.iter_mut().enumerate() {
            let mut value_total = 0.0;
            let pos = (i as f32) / ((self.len - 1) as f32);
            for sprite in sprites {
                let delta = (pos - sprite.get_pos()).abs();
                let value = (1.0 - (delta * sprite.get_falloff())).max(0.0) * 1.5;
                value_total = (value_total + value.min(1.0)).min(1.0);
            }
            *v = value_total;
        }
        match self.led_type {
            LedType::Apa102 => {
                self.buffer.chunks_mut(4).skip(1).enumerate().filter(|(i, _v)| { i < &self.len }).for_each(|(i, v)| {
                    let pos = (i as f32) / ((self.len - 1) as f32);
                    let mut pos_bipolar = pos * 2.0 - 1.0;
                    if pos_bipolar < 0.0 { pos_bipolar = pos_bipolar * -1.0 }
                    let gradient = hsv_interp(&col2, &col1, pos_bipolar);
                    let rgb = hsv_interp(&gradient, &col3, sprite_values[i]).to_rgb();
                    v[0] = 255;
                    v[1] = rgb.get_b();
                    v[2] = rgb.get_g();
                    v[3] = rgb.get_r();
                });
            }
            LedType::Ws2801 => {
                self.buffer.chunks_mut(3).enumerate().for_each(|(i, v)| {
                    let pos = (i as f32) / ((self.len - 1) as f32);
                    let mut pos_bipolar = pos * 2.0 - 1.0;
                    if pos_bipolar < 0.0 { pos_bipolar = pos_bipolar * -1.0 }
                    //let rgb = hsv_interp(&col2, &col1, pos_bipolar).to_rgb();
                    let gradient = hsv_interp(&col2, &col1, pos_bipolar);
                    let rgb = hsv_interp(&gradient, &col3, sprite_values[i]).to_rgb();
                    v[0] = rgb.get_r();
                    v[1] = rgb.get_g();
                    v[2] = rgb.get_b();
                });
            }
        }
    }

    /*
    pub fn fill_random(&mut self, start: &ColorHsv, end: &ColorHsv, sprites: &RandomSprites<{ NUM_LEDS }>) {
        match self.led_type {
            LedType::Apa102 => {
                self.buffer.chunks_mut(4).skip(1).zip(sprites.get_sprites()).for_each(|(led, sprite)| {
                    let rgb = hsv_interp(&end, &start, sprite.get_value()).to_rgb();
                    led[0] = 255;
                    led[1] = rgb.get_b();
                    led[2] = rgb.get_g();
                    led[3] = rgb.get_r();
                });
            }
            LedType::Ws2801 => {                
                self.buffer.chunks_mut(3).zip(sprites.get_sprites()).for_each(|(led, sprite)| {
                    let rgb = hsv_interp(&end, &start, sprite.get_value()).to_rgb();
                    led[0] = rgb.get_r();
                    led[1] = rgb.get_g();
                    led[2] = rgb.get_b();
                });
            }
        }
    }
    */
}
