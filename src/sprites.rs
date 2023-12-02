use fastrand;

#[derive(Copy, Clone, Debug)]
pub struct Sprite {
    pos: f32,
    falloff: f32,
    speed: f32,
    max_speed: f32,
}

impl Sprite {
    pub fn new(pos: f32, falloff: f32, mut speed: f32, max_speed: f32) -> Self {
        let max_speed_clip = max_speed.max(0.001);
        if speed == 0.0 { 
            speed = 0.0001; 
        } else if speed > 0.0 {
            speed = speed.max(0.0001).min(max_speed_clip);
        } else if speed < 0.0 {
            speed = speed.min(-0.0001).max(max_speed_clip * -1.0);
        }
        //if speed > 0.0 && speed < 0.0001 { speed = 0.0001 }
        //if speed < 0.0 && speed > -0.0001 { speed = -0.0001 }
        Self {
            pos: pos.max(0.0).min(1.0),
            falloff: falloff.max(1.0),
            speed: speed,
            max_speed: max_speed_clip,
        }
    }

    pub fn run(&mut self) {
        if self.speed > 0.0 {
            self.pos = self.pos + self.speed;
            if self.pos >= 1.0 {
                self.pos = 1.0;
                //self.speed = self.speed * -1.0;
                self.speed = (fastrand::f32() * self.max_speed).max(0.0001) * -1.0;
            }
        } else if self.speed < 0.0 {
            self.pos = self.pos + self.speed;
            if self.pos <= 0.0 {
                self.pos = 0.0;
                //self.speed = self.speed * -1.0;
                self.speed = (fastrand::f32() * self.max_speed).max(0.0001);
            }
        }
    }

    pub fn get_pos(&self) -> f32 {
        self.pos
    }

    pub fn get_falloff(&self) -> f32 {
        self.falloff
    }
}
