//~ use std::collections::HashMap;
use std::{env, process};
use std::fs::{self, File};
use std::io::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;

use fastrand;
use serde::{Serialize, Deserialize};
//~ use serde_derive::{Deserialize, Serialize};
use spidev::{Spidev, SpidevOptions, SpiModeFlags};
use warp::Filter;

mod colors;
use colors::{ColorRgb, ColorHsv};

mod leds;
use leds::{Leds, LedType};

mod sprites;
use sprites::Sprite;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct LedState {
    color1: Option<ColorHsv>,
    color2: Option<ColorHsv>,
    color3: Option<ColorHsv>,
    pattern: Option<u8>,
}

impl LedState {
    fn new() -> Self {
        LedState {
            color1: Some(ColorHsv::new(0.0, 0.0, 0.0)),
            color2: Some(ColorHsv::new(0.0, 0.0, 0.0)),
            color3: Some(ColorHsv::new(0.0, 0.0, 0.0)),
            pattern: Some(0),
        }
    }

    fn serialize(&self) -> [u8; 37] {
        let mut out = [0; 37];
        
        if self.color1.is_some() {
            let c1 = self.color1.unwrap().to_le_bytes();
            out.iter_mut().take(12).zip(c1).for_each(|(v, c)| *v = c );
        }
        if self.color2.is_some() {
            let c2 = self.color2.unwrap().to_le_bytes();
            out.iter_mut().skip(12).take(12).zip(c2).for_each(|(v, c)| *v = c );
        }
        if self.color3.is_some() {
            let c3 = self.color3.unwrap().to_le_bytes();
            out.iter_mut().skip(24).take(12).zip(c3).for_each(|(v, c)| *v = c );
        }
        if self.pattern.is_some() {
            out[36] = self.pattern.unwrap();
        }
        
        out
    }

    fn deserialize(bytes: [u8; 37]) -> Self {
        let mut c1 = [0; 12];
        let mut c2 = [0; 12];
        let mut c3 = [0; 12];
        c1.iter_mut().zip(bytes.iter().take(12)).for_each(|(i, v)| *i = *v );
        c2.iter_mut().zip(bytes.iter().skip(12).take(12)).for_each(|(i, v)| *i = *v );
        c3.iter_mut().zip(bytes.iter().skip(24).take(12)).for_each(|(i, v)| *i = *v );
        
        Self {
            color1: Some(ColorHsv::from_le_bytes(c1)),
            color2: Some(ColorHsv::from_le_bytes(c2)),
            color3: Some(ColorHsv::from_le_bytes(c3)),
            pattern: Some(bytes[36]),
        }
    }
}

fn brightness_adjust(leds: &LedState, brightness: f32) -> [ColorHsv; 3] {
    let mut col1 = leds.color1.unwrap();
    let mut col2 = leds.color2.unwrap();
    let mut col3 = leds.color3.unwrap();

    col1.set_v(col1.get_v() * brightness);
    col2.set_v(col2.get_v() * brightness);
    col3.set_v(col3.get_v() * brightness);
    [col1, col2, col3]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Success<'a> {
    msg: &'a str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Error<'a> {
    msg: &'a str,
}

#[derive(Deserialize)]
struct Config {
    main: Main,
    patterns: Patterns,
    rand: Rand,
}

#[derive(Deserialize)]
struct Main {
    led_count: Vec<usize>,
    led_type: u8,
    brightness: f32,
    secs_per_update: f32,
    port: u16,
    spi_devices: Vec<String>,
    devices: Vec<String>,
}

#[derive(Deserialize)]
struct Patterns {
    names: Vec<String>,
    scroll_speed: f32,
}

#[derive(Deserialize)]
struct Rand {
    count: usize,
    falloff: f32,
    max_speed: f32
}

#[tokio::main]
async fn main() {
    let config_path = env::args().nth(1).unwrap_or_else(|| {
        println!("no config file specified");
        process::exit(1);
    });
    let config_raw = fs::read_to_string(&config_path).unwrap_or_else(|err| {
        println!("error reading config: {}", err);
        process::exit(1);
    });
    let config: Config = toml::from_str(&config_raw).unwrap_or_else(|err| {
        println!("error parsing config: {}", err);
        process::exit(1);
    });
    
    //~ let led_state = Arc::new(Mutex::new(LedState::new()));
    let led_state: Arc<Mutex<Vec<LedState>>> = Arc::new(Mutex::new(Vec::new()));
    for (index, _device) in config.main.devices.iter().enumerate() {
        if let Ok(mut file) = File::open(format!("state.{}", index)) {
            let mut buf = [0; 37];
            if let Ok(_) = file.read_exact(&mut buf) {
                let state = LedState::deserialize(buf);
                led_state.lock().unwrap().push(state);
            } else {
                led_state.lock().unwrap().push(LedState::new());
            }
        } else {
            led_state.lock().unwrap().push(LedState::new());
        }
    }
    
    let led_state_inner = led_state.clone();
    let led_state = warp::any().map(move || led_state.clone());
    //~ let spi_device = config.main.spi_device.clone();

    let patterns = warp::any().map(move || config.patterns.names.clone());
    let devices = warp::any().map(move || config.main.devices.clone());
    //let brightness = warp::any().map(move || config.main.brightness.clone());
    
    thread::spawn(move || {
        let led_type = match config.main.led_type {
            0 => LedType::Apa102,
            1 => LedType::Ws2801,
            _ => LedType::Apa102,
        };
        //let mut leds_1 = Leds::new(config.main.led_count, led_type);
        let mut offset: f32 = 0.0;
        let mut sprites = Vec::with_capacity(config.rand.count);
        for _ in 0..config.rand.count {
            let pos = fastrand::f32();
            let speed = ((fastrand::f32() * 2.0) - 1.0) * config.rand.max_speed;
            sprites.push(Sprite::new(pos, config.rand.falloff, speed, config.rand.max_speed));
        }
        //println!("sprites: {:?}", &sprites);
        let mut leds = Vec::new();
        for count in config.main.led_count {
            //let mut leds_1 = Leds::new(count, led_type);
            leds.push(Leds::new(count, led_type.clone()));
        }
        let mut spi_devs = Vec::new();
        for i in config.main.spi_devices {
            //let mut spi = Spidev::open(&i).expect(&format!("Unable to open SPI device {}", &i));
            let spi = Spidev::open(&i);
            if spi.is_ok() {
                let mut spi_inner = spi.unwrap();
                let options = SpidevOptions::new()
                    .bits_per_word(8)
                    .max_speed_hz(8_000_000)
                    .mode(SpiModeFlags::SPI_MODE_0)
                    .build();
                //spi_inner.configure(&options).unwrap();
                if spi_inner.configure(&options).is_ok() {
                    spi_devs.push(Some(spi_inner));
                } else {
                    spi_devs.push(None);
                }
            } else {
                spi_devs.push(None);
            }
        }
        //let options = SpidevOptions::new()
        //    .bits_per_word(8)
        //    .max_speed_hz(8_000_000)
        //    .mode(SpiModeFlags::SPI_MODE_0)
        //    .build();
        //spi.configure(&options).unwrap();

        //println!("config.main.led_count: {}", config.main.led_count);
        loop {
            //~ println!("LED state: {:?}", led_state_inner);
            
            let leds_data = match led_state_inner.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            for ((led_data, mut led), mut spi) in leds_data.iter().zip(&mut leds).zip(&mut spi_devs) {
                match led_data.pattern {
                    // all LEDs off
                    Some(0) => {
                        led.all_off();
                        spi.iter_mut().for_each(|i| {
                            let _ = i.write(&led.get_buffer());
                        });
                    },
                    // fixed gradient
                    Some(1) => {
                        let cols = brightness_adjust(&led_data, config.main.brightness);
                        led.fill_gradient_triple(&cols[0], &cols[1], &cols[2]);
                        spi.iter_mut().for_each(|i| {
                            let _ = i.write(&led.get_buffer());
                        });
                    },
                    // scrolling sine wave
                    Some(2) => {
                        let cols = brightness_adjust(&led_data, config.main.brightness);
                        led.fill_sine(&cols[0], &cols[1], &cols[2], offset);
                        spi.iter_mut().for_each(|i| {
                            let _ = i.write(&led.get_buffer());
                        });
                    },
                    // random sprites
                    Some(3) => {
                        let cols = brightness_adjust(&led_data, config.main.brightness);
                        led.fill_sprites(&cols[0], &cols[1], &cols[2], &sprites);
                        spi.iter_mut().for_each(|i| {
                            let _ = i.write(&led.get_buffer());
                        });
                    },
                    Some(4) => {
                        println!("pattern4");
                        //~ let _ = spi.write(&leds_1.get_buffer());
                    },
                    Some(5) => {
                        println!("pattern5");
                        //~ let _ = spi.write(&leds_1.get_buffer());
                    },
                    _ => (),
                }
            }
            drop(leds_data);
            offset = (offset + config.patterns.scroll_speed) % 1.0;
            for i in &mut sprites {
                i.run();
                //println!("{:?}", &i);
            }
            
            sleep(Duration::from_secs_f32(config.main.secs_per_update));
        }
    });

    let index = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./static/index.html"));

    let style = warp::get()
        .and(warp::path("style.css"))
        .and(warp::fs::file("./static/style.css"));

    let pkg = warp::path("pkg")
        .and(warp::fs::dir("./wasm/pkg/"));
    
    let get = warp::path("get")
        .and(warp::path::param::<usize>())
        .and(led_state.clone())
        .map(|led_id: usize, leds_data: Arc<Mutex<Vec<LedState>>>| {
            let leds_data = match leds_data.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            if leds_data.get(led_id).is_some() {
                warp::reply::json(&leds_data[led_id])
            } else {
                warp::reply::json(&Error {msg: "invalid ID"})
            }
        });

    let set = warp::path("set")
        .and(warp::post())
        .and(warp::path::param::<usize>())
        .and(warp::body::content_length_limit(500))
        .and(warp::body::json())
        .and(led_state.clone())
        .map(|led_id: usize, post: LedState, leds_data: Arc<Mutex<Vec<LedState>>>| {
            let mut leds_data = match leds_data.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            if leds_data.get(led_id).is_some() {
                if post.color1.is_some() {
                    leds_data[led_id].color1 = post.color1;
                }
                if post.color2.is_some() {
                    leds_data[led_id].color2 = post.color2;
                }
                if post.color3.is_some() {
                    leds_data[led_id].color3 = post.color3;
                }
                if post.pattern.is_some() {
                    leds_data[led_id].pattern = post.pattern;
                }
                if let Ok(mut file) = File::create(format!("state.{}", led_id)) {
                    let _ = file.write_all(&leds_data[led_id].serialize());
                }
                warp::reply::json(&Success {msg: "OK"})
            } else {
                warp::reply::json(&Error {msg: "invalid ID"})
            }
        });

    let patterns = warp::path("patterns")
        .and(patterns.clone())
        .map(|v| {
            warp::reply::json(&v)
        });
    
    let devices = warp::path("devices")
        .and(devices.clone())
        .map(|v| {
            warp::reply::json(&v)
        });

    let routes = index
        .or(style)
        .or(pkg)
        .or(get)
        .or(set)
        .or(patterns)
        .or(devices);

    warp::serve(routes)
        .run(([0, 0, 0, 0], config.main.port))
        .await;
}
