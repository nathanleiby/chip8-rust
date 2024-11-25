use std::error::Error;

use interpreter::Interpreter;
use screen::Screen;

mod font;
mod interpreter;
mod screen;

const USAGE: &str = "usage: chip8-rust <file.ch8>";

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // read program
    let rom = std::env::args().nth(1).expect(USAGE);

    let screen = Screen::new();
    let mut interpreter = Interpreter::new(screen);

    interpreter.run(&rom)
}
