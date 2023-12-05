// build command:
// wasm-pack build --release --target web

use serde::{Serialize, Deserialize};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{JsFuture, spawn_local};

use web_sys::{console, HtmlInputElement, HtmlOptionElement, HtmlSelectElement, Request, RequestInit, RequestMode, Response, SvgElement};

struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Serialize, Deserialize)]
struct ColorHsv {
    h: f64,
    s: f64,
    v: f64,
}

impl ColorHsv {
    fn new(h: f64, s: f64, v: f64) -> Self {
        Self { 
            h: h.max(0.0).min(1.0), 
            s: s.max(0.0).min(1.0), 
            v: v.max(0.0).min(1.0),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct LedState {
    color1: ColorHsv,
    color2: ColorHsv,
    color3: ColorHsv,
    pattern: u8,
}

impl LedState {
    fn new(c1h: f64, c1s: f64, c1v: f64, c2h: f64, c2s: f64, c2v: f64, c3h: f64, c3s: f64, c3v: f64, p: u8) -> Self {
        LedState {
            color1: ColorHsv::new(c1h, c1s, c1v),
            color2: ColorHsv::new(c2h, c2s, c2v),
            color3: ColorHsv::new(c3h, c3s, c3v),
            pattern: p,
        }
    }
} 

#[derive(Serialize, Deserialize)]
struct Options (Vec<String>);

fn window() -> web_sys::Window {
    web_sys::window().unwrap()
}

fn document() -> web_sys::Document {
    window().document().unwrap()
}

fn get_value(id: &str) -> f64 {
    match document().get_element_by_id(id) {
        Some(v) => {
            match v.dyn_into::<HtmlInputElement>() {
                Ok(v) => v.value_as_number(),
                Err(_) => 0.0,
            }
        }
        None => 0.0,
    }
}

/*
fn get_element_text(id: &str) -> String {
    match document().get_element_by_id(id) {
        Some(v) => v.inner_html(),
        None => String::from(""),
    }
}
*/

fn get_svg(id: &str) -> SvgElement {
    document()
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<SvgElement>()
        .unwrap()
}

fn get_input(id: &str) -> HtmlInputElement {
    document()
        .get_element_by_id(id)
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap()
}

fn get_select_value(id: &str) -> u8 {
    match document().get_element_by_id(id) {
        Some(v) => {
            match v.dyn_into::<HtmlSelectElement>() {
                Ok(v) => {
                    match u8::from_str_radix(&v.value(), 10) {
                        Ok(v) => v,
                        Err(_) => 0,
                    }
                }
                Err(_) => 0,
            }
        }
        None => 0,
    }
}

fn set_pattern(id: &str, index: u8) {
    match document().get_element_by_id(id) {
        Some(element) => {
            match element.dyn_into::<HtmlSelectElement>() {
                Ok(select) => {
                    select.set_value(&format!("{}", index));
                }
                Err(_) => (),
            }
        }
        None => (),
    }
}

fn set_swatch(hue_id: &str, sat_id: &str, val_id: &str, swatch_id: &str) {
    let hue = get_value(hue_id);
    let sat = get_value(sat_id);
    let val = get_value(val_id);
    
    let swatch = get_svg(swatch_id);
    
    let rgb = hsv_2_rgb(&ColorHsv{ h: hue, s: sat, v: val });
    
    swatch.style().set_property("fill", &format!("rgba({}, {}, {}, 1.0)", rgb.r, rgb.g, rgb.b)).unwrap();
}

fn set_input(id: &str, val: &str) {
    let input = get_input(id);
    input.set_value(val);
}

fn set_input_f64(id: &str, val: f64) {
    let input = get_input(id);
    input.set_value(&format!("{}", val));
}

fn set_options(id: &str, options: Vec<String>) {
    match document().get_element_by_id(id) {
        Some(v) => {
            match v.dyn_into::<HtmlSelectElement>() {
                Ok(v) => {
                    for (index, value) in options.iter().enumerate() {
                        let opt = HtmlOptionElement::new_with_text_and_value(value, &format!("{}", index)).unwrap();
                        v.add_with_html_option_element(&opt).unwrap();
                    }
                }
                Err(_) => {},
            }
        }
        None => {},
    }
}

fn hsv_2_rgb(col: &ColorHsv) -> Pixel {
    let mut out = Pixel { r: 0, g: 0, b: 0 };
    let h_wrap = col.h.rem_euclid(1.0);
    
    match (h_wrap * 6.0).trunc() as u8 {
        0 => {
            out.r = (col.v * 255.0) as u8;
            out.g = ((col.v * (1.0 - col.s * (1.0 - ((col.h * 6.0) - ((col.h * 6.0).trunc()))))) * 255.0) as u8;
            out.b = ((col.v * (1.0 - col.s)) * 255.0) as u8;
        }
        1 => {
            out.r = ((col.v * (1.0 - col.s * ((col.h * 6.0) - ((col.h * 6.0).trunc())))) * 255.0) as u8;
            out.g = (col.v * 255.0) as u8;
            out.b = ((col.v * (1.0 - col.s)) * 255.0) as u8;
        }
        2 => {
            out.r = ((col.v * (1.0 - col.s)) * 255.0) as u8;
            out.g = (col.v * 255.0) as u8;
            out.b = ((col.v * (1.0 - col.s * (1.0 - ((col.h * 6.0) - ((col.h * 6.0).trunc()))))) * 255.0) as u8;
        }
        3 => {
            out.r = ((col.v * (1.0 - col.s)) * 255.0) as u8;
            out.g = ((col.v * (1.0 - col.s * ((col.h * 6.0) - ((col.h * 6.0).trunc())))) * 255.0) as u8;
            out.b = (col.v * 255.0) as u8;
        }
        4 => {
            out.r = ((col.v * (1.0 - col.s * (1.0 - ((col.h * 6.0) - ((col.h * 6.0).trunc()))))) * 255.0) as u8;
            out.g = ((col.v * (1.0 - col.s)) * 255.0) as u8;
            out.b = (col.v * 255.0) as u8;
        }
        5 => {
            out.r = (col.v * 255.0) as u8;
            out.g = ((col.v * (1.0 - col.s)) * 255.0) as u8;
            out.b = ((col.v * (1.0 - col.s * ((col.h * 6.0) - ((col.h * 6.0).trunc())))) * 255.0) as u8;
        }
        _ => (),
    }
    out
}

async fn get_set_options(path: &str) {
    let win_proto = window().location().protocol().expect("unable to get window().location().protocol()");
    let win_host = window().location().host().expect("unable to get window().location().host()");
    let win_url_base = format!("{}//{}", win_proto, win_host);
    let url = format!("{}/{}", &win_url_base, path);
    let mut options = Vec::new();
    let mut request_opts = RequestInit::new();
    request_opts.method("GET");
    request_opts.mode(RequestMode::Cors);    
    let request = Request::new_with_str_and_init(&url, &request_opts).expect("get_set_options() request failed");
    let response = JsFuture::from(window().fetch_with_request(&request)).await;
    match response {
        Ok(v) => {
            let resp: Response = v.dyn_into().unwrap();
            let resp_json = JsFuture::from(resp.json().unwrap()).await.unwrap();
            let resp_parsed: Result<Options, _> = serde_wasm_bindgen::from_value(resp_json);
            match resp_parsed {
                Ok(v) => {
                    for i in v.0 {
                        options.push(i);
                        //console::log_1(&value.into());
                    }
                }
                Err(e) => {console::log_1(&e.into())}
            }
        }
        Err(_e) => {}
    }
    set_options(path, options);
}

async fn get_leds() {
    let win_proto = window().location().protocol().expect("unable to get window().location().protocol()");
    let win_host = window().location().host().expect("unable to get window().location().host()");
    let win_url_base = format!("{}//{}", win_proto, win_host);
    let device = get_select_value("devices");
    let status_url = format!("{}/get/{}", &win_url_base, &device);
    let mut status_opts = RequestInit::new();
    status_opts.method("GET");
    status_opts.mode(RequestMode::Cors);
    let status_request = Request::new_with_str_and_init(&status_url, &status_opts).expect("new status_request failed");
    let status_response = JsFuture::from(window().fetch_with_request(&status_request)).await;
    match status_response {
        Ok(v) => {
            let resp: Response = v.dyn_into().unwrap();
            let resp_text_js = JsFuture::from(resp.json().unwrap()).await.unwrap();
            let resp_parsed: Result<LedState, _> = serde_wasm_bindgen::from_value(resp_text_js);
            match resp_parsed {
                Ok(v) => {
                    set_input_f64("color1_h", v.color1.h);
                    set_input_f64("color1_h_text", v.color1.h);

                    set_input_f64("color1_s", v.color1.s);
                    set_input_f64("color1_s_text", v.color1.s);

                    set_input_f64("color1_v", v.color1.v);
                    set_input_f64("color1_v_text", v.color1.v);
                    
                    set_swatch("color1_h", "color1_s", "color1_v", "color1_swatch");
                    
                    set_input_f64("color2_h", v.color2.h);
                    set_input_f64("color2_h_text", v.color2.h);

                    set_input_f64("color2_s", v.color2.s);
                    set_input_f64("color2_s_text", v.color2.s);

                    set_input_f64("color2_v", v.color2.v);
                    set_input_f64("color2_v_text", v.color2.v);

                    set_swatch("color2_h", "color2_s", "color2_v", "color2_swatch");

                    set_input_f64("color3_h", v.color3.h);
                    set_input_f64("color3_h_text", v.color3.h);

                    set_input_f64("color3_s", v.color3.s);
                    set_input_f64("color3_s_text", v.color3.s);

                    set_input_f64("color3_v", v.color3.v);
                    set_input_f64("color3_v_text", v.color3.v);

                    set_swatch("color3_h", "color3_s", "color3_v", "color3_swatch");

                    set_pattern("patterns", v.pattern);

                },
                Err(e) => { console::log_1(&format!("failed to parse JSON: {:?}", e).into()) },
            }
        }
        Err(_e) => (),
    }
}

async fn set_leds() {
    {
        let win_proto = window().location().protocol().unwrap();
        let win_host = window().location().host().unwrap();
        let win_url_base = format!("{}//{}", win_proto, win_host);
        let device = get_select_value("devices");
        let url = format!("{}/set/{}", &win_url_base, &device);

        let mut req_opts = RequestInit::new();
        req_opts.method("POST");
        req_opts.mode(RequestMode::Cors);
        
        let color1_h = get_value("color1_h");
        let color1_s = get_value("color1_s");
        let color1_v = get_value("color1_v");
        
        let color2_h = get_value("color2_h");
        let color2_s = get_value("color2_s");
        let color2_v = get_value("color2_v");
        
        let color3_h = get_value("color3_h");
        let color3_s = get_value("color3_s");
        let color3_v = get_value("color3_v");
        
        let pattern = get_select_value("patterns");

        let state = LedState::new(color1_h, color1_s, color1_v, color2_h, color2_s, color2_v, color3_h, color3_s, color3_v, pattern);
        
        //let body = serde_wasm_bindgen::to_value(&state).unwrap();
        //req_opts.body(Some(&body));

        let body_string = format!(r#"{{"color1": {{"h": {}, "s": {}, "v": {}}}, "color2": {{"h": {}, "s": {}, "v": {}}}, "color3": {{"h": {}, "s": {}, "v": {}}}, "pattern": {}}}"#, 
            &state.color1.h, &state.color1.s, &state.color1.v,
            &state.color2.h, &state.color2.s, &state.color2.v,
            &state.color3.h, &state.color3.s, &state.color3.v,
            &state.pattern,
        );
        req_opts.body(Some(&JsValue::from_str(&body_string)));

        //console::log_1(body_string.into());
        //console::log_1(&body.as_string().unwrap().into());
        //console::log_1(&JsValue::from_str(&body_string));
        
        let request = Request::new_with_str_and_init(&url, &req_opts).unwrap();
        request.headers().set("Content-Type", "application/json").unwrap();
        
        let resp_value = JsFuture::from(window().fetch_with_request(&request)).await;
        match resp_value {
            Ok(_v)  => (),
            Err(_e) => (),
        }
    }
}

#[wasm_bindgen(start)]
pub async fn run() -> Result<(), JsValue> {
    let select_html = document()
        .get_elements_by_class_name("slider");
    let color_ids = [
        ["color1_h", "color1_s", "color1_v", "color1_swatch"],
        ["color2_h", "color2_s", "color2_v", "color2_swatch"],
        ["color3_h", "color3_s", "color3_v", "color3_swatch"]
    ];
    for i in 0..select_html.length() {
        let item = select_html.item(i).unwrap();
        let event_id = item.id();
        //let event_id_clone = event_id.clone();
        let event_text_id = format!("{}_text", event_id);
        let event_text_id_clone = event_text_id.clone();
        let mut color_id = None;
        for i in color_ids {
            if i.iter().any(|&x| x == event_id) {
                color_id = Some(i.clone());
            }
        }
        if color_id.is_some() { 
            let color_id_inner = color_id.unwrap();
            let callback = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let target = event.target().expect("unable to get Event target");
                if let Ok(input) = target.dyn_into::<HtmlInputElement>() {
                    set_input(&event_text_id, &input.value());
                    set_swatch(color_id_inner[0], color_id_inner[1], color_id_inner[2], color_id_inner[3]);
                }
            }) as Box<dyn FnMut(_)>);
            item.add_event_listener_with_callback("change", callback.as_ref().unchecked_ref())?;
            callback.forget();

            let text_item = document()
                .get_element_by_id(&event_text_id_clone)
                .expect("unable to get color text element");
            let text_callback = Closure::wrap(Box::new(move |event: web_sys::Event| {
                let target = event.target().expect("unable to get Event target");
                if let Ok(input) = target.dyn_into::<HtmlInputElement>() {
                    //console::log_1(&format!("slider event ID: {} | value: {}", input.id(), input.value()).into());
                    set_input(&event_id, &input.value());
                    set_swatch(color_id_inner[0], color_id_inner[1], color_id_inner[2], color_id_inner[3]);
                }
            }) as Box<dyn FnMut(_)>);
            text_item.add_event_listener_with_callback("change", text_callback.as_ref().unchecked_ref())?;
            text_callback.forget();
        }
    }

    let device_el = document()
        .get_element_by_id("devices")
        .expect("unable to get devices element");
    let device_el_callback = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
        {
            spawn_local(async {
                get_leds().await;
            });
        }
    }) as Box<dyn FnMut(_)>);
    device_el.add_event_listener_with_callback("change", device_el_callback.as_ref().unchecked_ref())?;
    device_el_callback.forget();
    
    let send_btn = document()
        .get_element_by_id("send")
        .expect("unable to get send element");
    let send_btn_callback = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
        {
            spawn_local(async {
                set_leds().await;
            });
        }
    }) as Box<dyn FnMut(_)>);
    send_btn.add_event_listener_with_callback("mousedown", send_btn_callback.as_ref().unchecked_ref())?;
    send_btn_callback.forget();
    
    get_set_options("patterns").await;
    get_set_options("devices").await;
    get_leds().await;
    
    Ok(())
}
