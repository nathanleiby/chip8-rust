use std::env;
use std::error::Error;
use std::io::Read;

use interpreter::Interpreter;

mod font;
mod interpreter;

use macroquad::prelude::*;

use macroquad::{
    color::Color,
    input::{is_key_released, KeyCode},
    window::{next_frame, Conf},
};

const SCALE: f32 = 16.;

const WINDOW_WIDTH: f32 = 64. * SCALE;
const WINDOW_HEIGHT: f32 = 32. * SCALE;

fn conf() -> Conf {
    #[allow(clippy::cast_possible_truncation)]
    Conf {
        window_title: String::from("Chip 8"),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        high_dpi: true,
        ..Default::default()
    }
}

const INSTRUCTIONS_PER_LOOP: usize = 5;

fn capture_keyboard_input(interpreter: &mut Interpreter) {
    for (idx, k) in [
        // this order relates to the original layout of the Chip-8 Keyboard
        KeyCode::X,    // 0
        KeyCode::Key1, // 1
        KeyCode::Key2, // 2
        KeyCode::Key3, // 3
        KeyCode::Q,    // 4
        KeyCode::W,    // 5
        KeyCode::E,    // 6
        KeyCode::A,    // 7
        KeyCode::S,    // 8
        KeyCode::D,    // 9
        KeyCode::Z,    // A
        KeyCode::C,    // B
        KeyCode::Key4, // C
        KeyCode::R,    // D
        KeyCode::F,    // E
        KeyCode::V,    // F
    ]
    .iter()
    .enumerate()
    {
        if is_key_down(*k) {
            interpreter.set_key(idx, true);
        } else {
            interpreter.set_key(idx, false);
        }
    }
}

fn update_display(interpreter: &Interpreter, pixel_brightness: &mut [f32; 64 * 32]) {
    for (idx, on) in interpreter.pixels().iter().enumerate() {
        if *on {
            pixel_brightness[idx] += 0.25;
            pixel_brightness[idx] = clamp(pixel_brightness[idx], 0., 1.);
        } else {
            // fade out
            pixel_brightness[idx] -= 0.05;
            pixel_brightness[idx] = clamp(pixel_brightness[idx], 0., 1.);
        }
    }
    for (idx, brightness) in pixel_brightness.iter().enumerate() {
        let row = (idx / 64) as f32;
        let col = (idx % 64) as f32;
        let red = Color::from_hex(0xA4193D);
        let tan = Color::from_hex(0xFFDFB9);
        let color = Color::from_rgba(
            ((red.r * brightness + tan.r * (1. - brightness)) / 2. * 255.) as u8,
            ((red.g * brightness + tan.g * (1. - brightness)) / 2. * 255.) as u8,
            ((red.b * brightness + tan.b * (1. - brightness)) / 2. * 255.) as u8,
            255,
        );

        draw_rectangle(col * SCALE, row * SCALE, 1.0 * SCALE, 1.0 * SCALE, color);
    }
}

const PONG_ROM: &[u8; 246] = include_bytes!(".././assets/roms/PONG");

#[macroquad::main(conf)]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    let mut interpreter = Interpreter::new();
    // if a rom is given, load that. Else load PONG
    let rom = std::env::args().nth(1);
    if let Some(rom) = rom {
        // read file
        let mut rom_file = std::fs::File::open(rom)?;
        let mut rom_bytes = Vec::new();
        rom_file.read_to_end(&mut rom_bytes)?;
        interpreter.load_program(&rom_bytes);
    } else {
        interpreter.load_program(PONG_ROM);
    }

    // let rom = std::env::args().nth(1).expect(USAGE);
    #[cfg(target_arch = "wasm32")]
    wasm_logger::init(wasm_logger::Config::default());

    // let mut should_step = false;

    // TODO: sound?
    let mut pixel_brightness: [f32; 64 * 32] = [0.; 64 * 32];

    loop {
        if is_key_down(KeyCode::LeftShift) && is_key_released(KeyCode::Escape) {
            break;
        }

        // // TODO: temporarily for debugging.. we require pressing Space to step forward
        // if is_key_pressed(KeyCode::Space) {
        //     should_step = true;
        // }

        // expose current state (visuals, audio)
        update_display(&interpreter, &mut pixel_brightness);
        // TODO: play sound, if appropriate

        // capture changes
        capture_keyboard_input(&mut interpreter);
        interpreter.decrement_timers(); // assumes game loop is running at approx 60fps

        for _ in 0..INSTRUCTIONS_PER_LOOP {
            // if should_step {
            interpreter.step()?;
            // should_step = false;
            // }
        }

        next_frame().await;
    }

    Ok(())
}
