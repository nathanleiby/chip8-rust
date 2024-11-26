use std::error::Error;

use interpreter::Interpreter;

mod font;
mod interpreter;

const USAGE: &str = "usage: chip8-rust <file.ch8>";

use macroquad::prelude::*;

use macroquad::{
    color::Color,
    input::{is_key_released, KeyCode},
    window::{next_frame, Conf},
};

pub const CARROT_ORANGE: Color = Color {
    r: 247.0 / 255.0,
    g: 152.0 / 255.0,
    b: 36.0 / 255.0,
    a: 1.0,
};
pub const GUNMETAL: Color = Color {
    r: 49.0 / 255.0,
    g: 57.0 / 255.0,
    b: 60.0 / 255.0,
    a: 1.0,
};

const WINDOW_WIDTH: f32 = 1366.0;
const WINDOW_HEIGHT: f32 = 768.0;

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

const SCALE: f32 = 16.;
const INSTRUCTIONS_PER_STEP: usize = 5;

#[macroquad::main(conf)]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // read program
    let rom = std::env::args().nth(1).expect(USAGE);
    // TODO: keyboard?
    // TODO: sound?

    let mut interpreter = Interpreter::new();
    interpreter.read_program_from_file(&rom)?;

    // let mut should_step = false;

    loop {
        if is_key_down(KeyCode::LeftShift) && is_key_released(KeyCode::Escape) {
            break;
        }

        // // TODO: temporarily for debugging.. we require pressing Space to step forward
        // if is_key_pressed(KeyCode::Space) {
        //     should_step = true;
        // }

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

        for (idx, on) in interpreter.pixels().iter().enumerate() {
            let row = (idx / 64) as f32;
            let col = (idx % 64) as f32;
            let color = if *on {
                // Color32::from_rgb(1, 0, 0)
                RED
            } else {
                BLUE
            };
            draw_rectangle(col * SCALE, row * SCALE, 1.0 * SCALE, 1.0 * SCALE, color);
        }

        for _ in 0..INSTRUCTIONS_PER_STEP {
            // if should_step {
            interpreter.step()?;
            // should_step = false;
            // }
        }

        next_frame().await;
    }

    Ok(())
}
