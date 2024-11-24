mod font;

use std::{
    env::args,
    fs::File,
    io::{BufReader, Error, Read},
    path::Path,
};

use font::FONT;

// wrap u8 for now
type u4 = u8;

// wrap u16 for now
type u12 = u16;

#[derive(Debug)]
enum Op {
    // NYI, // Not yet implemented. For use during development of Chip8 Interpreter
    CLS,
    RET,
    SYS { addr: u12 },
    JP { addr: u12 },
    CALL { addr: u12 },
    SE { x: u4, byte: u8 },
    SNE { x: u4, byte: u8 },
    SE2 { x: u4, y: u4 },
    LD { x: u4, byte: u8 },
    ADD { x: u4, byte: u8 },
    LD_VX_VY { x: u4, y: u4 },
    OR_VX_VY { x: u4, y: u4 },
    AND_VX_VY { x: u4, y: u4 },
    XOR_VX_VY { x: u4, y: u4 },
    ADD_VX_VY { x: u4, y: u4 },
    SUB_VX_VY { x: u4, y: u4 },
    SHR_VX_VY { x: u4, y: u4 },
    SUBN_VX_VY { x: u4, y: u4 },
    SHL_VX_VY { x: u4, y: u4 },
    SNE_VX_VY { x: u4, y: u4 },
    LD_I { addr: u12 },
    JP_V0 { addr: u12 },
    RND { x: u4, byte: u8 },
    DRW { x: u4, y: u4, nibble: u4 },
    SKP { x: u4 },
    SKNP { x: u4 },
    LD_VX_DT { x: u4 },
    LD_VX_K { x: u4 },
    LD_DT_VX { x: u4 },
    LD_ST_VX { x: u4 },
    ADD_I_VX { x: u4 },
    LD_F_VX { x: u4 },
    LD_B_VX { x: u4 },
    LD_I_VX { x: u4 },
    LD_VX_I { x: u4 },
    INVALID,
}

const MEMORY_SIZE: usize = 4096;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

type Pixels = [bool; SCREEN_WIDTH * SCREEN_HEIGHT];

struct Interpreter {
    memory_map: [u8; MEMORY_SIZE],
    _program_size: usize,
    program_counter: u16,

    stack_pointer: u8,
    stack: [u16; 16],

    registers: [u8; 16], // also called Vx
    // TODO: Is it possible in rust to make `vf` look like a field, even though it would be a method that maps back to these registers? e.g. (self.vf = 5) or (x = self.vf) would both work
    index_register: u16, // usually only stores lowest 12 bits, for memory addresses

    pixels: Pixels,

    /// "hardware" abstractions
    /// input: for the keyboard. represents whether key i is pressed
    keys: [bool; 16],
    /// screen: renders the pixels
    screen: Screen,
}

const FONT_START: usize = 0x50;
const PROGRAM_START: usize = 512;

impl Interpreter {
    fn new(screen: Screen) -> Self {
        // initialize memory map
        let mut memory_map = [0; 4096];
        // write font
        for (idx, c) in FONT.iter().enumerate() {
            memory_map[FONT_START + idx] = *c;
        }

        Interpreter {
            memory_map,
            _program_size: 0,

            stack: [0; 16],

            registers: [0; 16],
            index_register: 0,

            program_counter: PROGRAM_START as u16,
            stack_pointer: 0,

            pixels: [false; 64 * 32],

            keys: [false; 16],
            screen,
        }
    }

    /// Reads a program from a file and writes it into the memory_map
    fn read_program_from_file(&mut self, p: &str) -> Result<(), Error> {
        let mut file = File::open(p)?;

        let mut buffer = [0 as u8; 4096 - 512];
        self._program_size = file.read(&mut buffer)?;
        for (idx, b) in buffer.iter().enumerate() {
            self.memory_map[PROGRAM_START + idx] = *b;
        }

        Ok(())
    }

    fn can_continue(&self) -> bool {
        let is_within_memory = self.program_counter < MEMORY_SIZE as u16;
        let is_in_program = self.program_counter as usize <= PROGRAM_START + self._program_size;

        is_within_memory && is_in_program
    }

    fn print_program(&self) {
        for i in (PROGRAM_START..PROGRAM_START + self._program_size).step_by(2) {
            let inst = self.fetch_instruction_at(i);
            println!("addr={}  inst={:#06x}  op={:?}", i, inst, self.decode(inst));
        }
        println!("Program Size = {}", self._program_size);
    }

    fn fetch_instruction_at(&self, pc: usize) -> u16 {
        let first = self.memory_map[pc];
        let second = self.memory_map[pc + 1];
        let instruction = ((first as u16) << 8) | second as u16;
        instruction
    }

    fn fetch(&mut self) -> u16 {
        // gets the next two bytes and sets the program counter forward
        let instruction = self.fetch_instruction_at(self.program_counter as usize);
        self.program_counter += 2;
        instruction
    }

    fn decode(&self, instruction: u16) -> Op {
        let first_nibble: u4 = (((0xF000 as u16) & instruction) >> 12) as u4;
        let second_nibble = (((0x0F00 as u16) & instruction) >> 8) as u4;
        let third_nibble = (((0x00F0 as u16) & instruction) >> 4) as u4;
        let fourth_nibble = ((0x000F as u16) & instruction) as u4;

        let twelve_bits: u12 = (0x0FFF as u16) & instruction;
        let second_byte = ((0x00FF as u16) & instruction) as u8;
        println!(
            "instruction: {:#06x}, as nibbles: {:#03x} {:#03x} {:#03x} {:#03x}, second byte: {:#04x}, twelve_bits: {:#05x}",
            instruction, first_nibble, second_nibble, third_nibble, fourth_nibble, second_byte, twelve_bits
        );
        match first_nibble {
            0 => match instruction {
                0x00E0 => Op::CLS,
                0x00EE => Op::RET,
                _ => Op::SYS { addr: twelve_bits },
            },
            1 => Op::JP { addr: twelve_bits },
            2 => Op::CALL { addr: twelve_bits },
            3 => Op::SE {
                x: second_nibble,
                byte: second_byte,
            },
            4 => Op::SNE {
                x: second_nibble,
                byte: second_byte,
            },
            5 => {
                if fourth_nibble != 0 {
                    return Op::INVALID;
                }
                Op::SE2 {
                    x: second_nibble,
                    y: third_nibble,
                }
            }
            6 => Op::LD {
                x: second_nibble,
                byte: second_byte,
            },
            7 => Op::ADD {
                x: second_nibble,
                byte: second_byte,
            },
            8 => {
                let x = second_nibble;
                let y = third_nibble;

                match fourth_nibble {
                    0 => Op::LD_VX_VY { x, y },
                    1 => Op::OR_VX_VY { x, y },
                    2 => Op::AND_VX_VY { x, y },
                    3 => Op::XOR_VX_VY { x, y },
                    4 => Op::ADD_VX_VY { x, y },
                    5 => Op::SUB_VX_VY { x, y },
                    6 => Op::SHR_VX_VY { x, y },
                    7 => Op::SUBN_VX_VY { x, y },
                    0xE => Op::SHL_VX_VY { x, y },
                    _ => Op::INVALID,
                }
            }
            9 => {
                if fourth_nibble != 0 {
                    return Op::INVALID;
                }

                let x = second_nibble;
                let y = third_nibble;
                Op::SNE_VX_VY { x, y }
            }
            0xA => Op::LD_I { addr: twelve_bits },
            0xB => Op::JP_V0 { addr: twelve_bits },
            0xC => Op::RND {
                x: second_nibble,
                byte: second_byte,
            },
            0xD => Op::DRW {
                x: second_nibble,
                y: third_nibble,
                nibble: fourth_nibble,
            },
            0xE => match second_byte {
                0x9E => Op::SKP { x: second_nibble },
                0xA1 => Op::SKNP { x: second_nibble },
                _ => Op::INVALID,
            },
            0xF => match second_byte {
                0x07 => Op::LD_VX_DT { x: second_nibble },
                0x0A => Op::LD_VX_K { x: second_nibble },
                0x15 => Op::LD_DT_VX { x: second_nibble },
                0x18 => Op::LD_ST_VX { x: second_nibble },
                0x1E => Op::ADD_I_VX { x: second_nibble },
                0x29 => Op::LD_F_VX { x: second_nibble },
                0x33 => Op::LD_B_VX { x: second_nibble },
                0x55 => Op::LD_I_VX { x: second_nibble },
                0x65 => Op::LD_VX_I { x: second_nibble },
                _ => Op::INVALID,
            },
            _ => Op::INVALID,
        }
    }

    fn execute(&mut self, op: Op) -> Result<(), Error> {
        match op {
            Op::CLS => self.screen.clear_screen(),
            Op::RET => todo!(),
            Op::SYS { addr: _ } => (),
            Op::JP { addr } => {
                log::debug!("jump to addr: {:#05x}", addr);
                self.program_counter = addr;
            }
            Op::CALL { addr } => todo!(),
            Op::SE { x, byte } => todo!(),
            Op::SNE { x, byte } => todo!(),
            Op::SE2 { x, y } => todo!(),
            Op::LD { x, byte } => self.registers[x as usize] = byte,
            Op::ADD { x, byte } => self.registers[x as usize] += byte,
            Op::LD_VX_VY { x, y } => self.registers[x as usize] = self.registers[y as usize],
            Op::OR_VX_VY { x, y } => todo!(),
            Op::AND_VX_VY { x, y } => todo!(),
            Op::XOR_VX_VY { x, y } => todo!(),
            Op::ADD_VX_VY { x, y } => todo!(),
            Op::SUB_VX_VY { x, y } => todo!(),
            Op::SHR_VX_VY { x, y } => todo!(),
            Op::SUBN_VX_VY { x, y } => todo!(),
            Op::SHL_VX_VY { x, y } => todo!(),
            Op::SNE_VX_VY { x, y } => todo!(),
            Op::LD_I { addr } => {
                self.index_register = addr;
            }
            Op::JP_V0 { addr } => {
                self.program_counter = addr + self.registers[0] as u16;
            }
            Op::RND { x, byte } => todo!(),
            Op::DRW { x, y, nibble } => {
                let x = self.registers[x as usize];
                let y = self.registers[y as usize];

                // read nibble bytes from register addrs
                let mut bytes_to_draw: Vec<u8> = vec![];
                for i in 0..nibble {
                    bytes_to_draw.push(self.memory_map[(self.index_register + i as u16) as usize]);
                }

                let min_row = y as usize;
                let max_row = y as usize + bytes_to_draw.len() - 1;
                for row_idx in min_row..=max_row {
                    let b = bytes_to_draw[row_idx - y as usize];
                    for bit_idx in (0..8).rev() {
                        let pixel_on = (b & 0x1 << bit_idx) > 0;
                        let pixel_pos = row_idx * SCREEN_WIDTH + x as usize + (7 - bit_idx);
                        self.pixels[pixel_pos] = pixel_on;
                        // TODO: handle collision flag
                    }
                }

                // // to convert to pixels to draw
                // let mut collision_flag = false;
                // for b in bytes_to_draw {
                //     for bit_idx in 7..=0 {
                //         let pixel_on = (b & 0x1 << bit_idx) > 0;
                //         self.pixels
                //     }
                // }
                // if collision_flag {
                //     // TODO: When does the overflow flag get set to false? Should I set to false if there's no overflow?
                //     self.registers[0xf] = 0x1; // true
                // }
                self.screen.draw(self.pixels);
            }
            Op::SKP { x } => {
                let is_key_pressed = self.keys[self.registers[x as usize] as usize];
                if is_key_pressed {
                    self.program_counter += 2;
                }
            }
            Op::SKNP { x } => {
                // skip if key not pressed
                let is_key_pressed = self.keys[self.registers[x as usize] as usize];
                if !is_key_pressed {
                    self.program_counter += 2;
                }
            }
            Op::LD_VX_DT { x } => todo!(),
            Op::LD_VX_K { x } => todo!(),
            Op::LD_DT_VX { x } => todo!(),
            Op::LD_ST_VX { x } => todo!(),
            Op::ADD_I_VX { x } => todo!(),
            Op::LD_F_VX { x } => todo!(),
            Op::LD_B_VX { x } => todo!(),
            Op::LD_I_VX { x } => todo!(),
            Op::LD_VX_I { x } => todo!(),
            Op::INVALID => todo!("this will aways fail"),
        }

        Ok(())
    }
}

struct Screen {}

impl Screen {
    pub fn new() -> Self {
        Screen {}
    }

    fn clear_screen(&self) {
        print!("{}[2J", 27 as char);
    }

    fn draw(&self, pixels: Pixels) {
        println!("Screen:");
        for row in pixels.chunks(SCREEN_WIDTH) {
            let s: String = row.iter().map(|x| if *x { "#" } else { "." }).collect();
            println!("{}", s);
        }
    }
}
const USAGE: &str = "usage: chip8-rust <file.ch8>";
fn main() -> Result<(), Error> {
    let mut screen = Screen::new();
    let mut interpreter = Interpreter::new(screen);

    // read program
    let rom = std::env::args().nth(1).expect(USAGE);
    interpreter.read_program_from_file(&rom)?;
    interpreter.print_program();

    // run the program
    println!("running the program...");
    // TODO: Consider instructions-per-sec
    //  "In practice, a standard speed of around 700 CHIP-8 instructions per second fits well enough for most CHIP-8 programs you’ll find"
    while interpreter.can_continue() {
        println!("pc: {:?}", interpreter.program_counter);
        let instruction = interpreter.fetch();
        let op = interpreter.decode(instruction);
        println!("op: {:?}", op);
        println!("registers (before): {:?}", interpreter.registers);
        interpreter.execute(op)?;
        println!("registers (after):  {:?}", interpreter.registers);

        // TODO: simplify
        println!("Press ENTER to continue...");
        let buffer = &mut [0u8];
        std::io::stdin().read_exact(buffer).unwrap();
    }

    println!("done");
    Ok(())
}
