use crate::interpreter::{Pixels, SCREEN_WIDTH};

pub struct Screen {}

impl Screen {
    pub fn new() -> Self {
        Screen {}
    }

    pub fn clear_screen(&self) {
        print!("{}[2J", 27 as char);
    }

    pub fn draw(&self, pixels: Pixels) {
        println!("Screen:");
        let margin_tb = "@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@";
        println!("{}", margin_tb);
        for row in pixels.chunks(SCREEN_WIDTH) {
            let s: String = row.iter().map(|x| if *x { "#" } else { " " }).collect();
            println!("@{}@", s);
        }
        println!("{}", margin_tb);
    }
}
