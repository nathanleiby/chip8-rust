use std::{error::Error, fs::File, io::Read};

use rand::Rng;

use crate::font::FONT;

// wrap u8 for now
type U4 = u8;

// wrap u16 for now
type U8 = u16;

#[derive(Debug)]
enum Op {
    Cls,
    Ret,
    Sys,
    Jp { nnn: U8 },
    Call { nnn: U8 },
    Se { x: U4, nn: u8 },
    Sne { x: U4, nn: u8 },
    SeVxVy { x: U4, y: U4 },
    Ld { x: U4, nn: u8 },
    Add { x: U4, nn: u8 },
    LdVxVy { x: U4, y: U4 },
    OrVxVy { x: U4, y: U4 },
    AndVxVy { x: U4, y: U4 },
    XorVxVy { x: U4, y: U4 },
    AddVxVy { x: U4, y: U4 },
    SubVxVy { x: U4, y: U4 },
    ShrVxVy { x: U4 }, // y: N },
    SubnVxVy { x: U4, y: U4 },
    ShlVxVy { x: U4 }, //  y: N },
    SneVxVy { x: U4, y: U4 },
    LdI { nnn: U8 },
    JpV0 { nnn: U8 },
    Rnd { x: U4, nn: u8 },
    Drw { x: U4, y: U4, n: U4 },
    Skp { x: U4 },
    Sknp { x: U4 },
    LdVxDt { x: U4 },
    LdVxK { x: U4 },
    LdDtVx { x: U4 },
    LdStVx { x: U4 },
    AddIVx { x: U4 },
    LdFVx { x: U4 },
    LdBVx { x: U4 },
    LdIVx { x: U4 },
    LdVxI { x: U4 },
    Invalid,
}

const MEMORY_SIZE: usize = 4096;

pub const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

pub type Pixels = [bool; SCREEN_WIDTH * SCREEN_HEIGHT];

pub struct Interpreter {
    memory_map: [u8; MEMORY_SIZE],
    _program_size: usize,
    program_counter: u16,

    stack_pointer: u8,
    stack: [u16; 16],

    registers: [u8; 16], // also called Vx
    // TODO: Is it possible in rust to make `vf` look like a field, even though it would be a method that maps back to these registers? e.g. (self.vf = 5) or (x = self.vf) would both work
    index_register: u16, // usually only stores lowest 12 bits, for memory addresses

    delay_timer: u8,
    sound_timer: u8,

    /// "hardware" abstractions
    /// input: for the keyboard. represents whether key i is pressed
    keys: [bool; 16],
    pixels: Pixels,
}

const FONT_START: usize = 0x50;
const PROGRAM_START: usize = 512;

impl Interpreter {
    pub fn new() -> Self {
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

            delay_timer: 0,
            sound_timer: 0,

            pixels: [false; 64 * 32],

            keys: [false; 16],
        }
    }

    pub fn set_key(&mut self, key_idx: usize, is_down: bool) {
        self.keys[key_idx] = is_down;
    }

    pub fn step(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.can_continue() {
            // exit early
            return Ok(());
        }

        log::debug!("pc: {:?}", self.program_counter);
        let instruction = self.fetch();
        let op = self.decode(instruction);
        log::debug!("op: {:?}", op);
        log::debug!("registers (before): {:?}", self.registers);
        self.execute(op)?;
        log::debug!("registers (after):  {:?}", self.registers);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        Ok(())
    }

    /// Reads a program from a file and writes it into the memory_map
    pub fn read_program_from_file(&mut self, p: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::open(p)?;

        let mut buffer = [0_u8; 4096 - 512];
        self._program_size = file.read(&mut buffer)?;
        for (idx, b) in buffer.iter().enumerate() {
            self.memory_map[PROGRAM_START + idx] = *b;
        }

        Ok(())
    }

    pub fn pixels(&self) -> Pixels {
        self.pixels
    }

    fn can_continue(&self) -> bool {
        let is_within_memory = self.program_counter < MEMORY_SIZE as u16;
        let is_in_program = self.program_counter as usize <= PROGRAM_START + self._program_size;

        is_within_memory && is_in_program
    }

    fn fetch_instruction_at(&self, pc: usize) -> u16 {
        let first = self.memory_map[pc];
        let second = self.memory_map[pc + 1];

        ((first as u16) << 8) | second as u16
    }

    fn fetch(&mut self) -> u16 {
        // gets the next two bytes and sets the program counter forward
        let instruction = self.fetch_instruction_at(self.program_counter as usize);
        self.program_counter += 2;
        instruction
    }

    fn decode(&self, instruction: u16) -> Op {
        let first_nibble: U4 = ((0xF000_u16 & instruction) >> 12) as U4;
        let x = ((0x0F00_u16 & instruction) >> 8) as U4;
        let y = ((0x00F0_u16 & instruction) >> 4) as U4;
        let n = (0x000F_u16 & instruction) as U4;

        let nnn: U8 = 0x0FFF_u16 & instruction;
        let nn = (0x00FF_u16 & instruction) as u8;
        log::debug!(
            "instruction: {:#06x}, as nibbles: {:#03x} {:#03x} {:#03x} {:#03x}, nn: {:#04x}, nnn: {:#05x}",
            instruction, first_nibble, x, y, n, nn, nnn
        );
        match first_nibble {
            0 => match instruction {
                0x00E0 => Op::Cls,
                0x00EE => Op::Ret,
                _ => Op::Sys,
            },
            1 => Op::Jp { nnn },
            2 => Op::Call { nnn },
            3 => Op::Se { x, nn },
            4 => Op::Sne { x, nn },
            5 => {
                if n != 0 {
                    return Op::Invalid;
                }
                Op::SeVxVy { x, y }
            }
            6 => Op::Ld { x, nn },
            7 => Op::Add { x, nn },
            8 => {
                let x = x;
                let y = y;

                match n {
                    0 => Op::LdVxVy { x, y },
                    1 => Op::OrVxVy { x, y },
                    2 => Op::AndVxVy { x, y },
                    3 => Op::XorVxVy { x, y },
                    4 => Op::AddVxVy { x, y },
                    5 => Op::SubVxVy { x, y },
                    6 => Op::ShrVxVy { x },
                    7 => Op::SubnVxVy { x, y },
                    0xE => Op::ShlVxVy { x },
                    _ => Op::Invalid,
                }
            }
            9 => {
                if n != 0 {
                    return Op::Invalid;
                }

                let x = x;
                let y = y;
                Op::SneVxVy { x, y }
            }
            0xA => Op::LdI { nnn },
            0xB => Op::JpV0 { nnn },
            0xC => Op::Rnd { x, nn },
            0xD => Op::Drw { x, y, n },
            0xE => match nn {
                0x9E => Op::Skp { x },
                0xA1 => Op::Sknp { x },
                _ => Op::Invalid,
            },
            0xF => match nn {
                0x07 => Op::LdVxDt { x },
                0x0A => Op::LdVxK { x },
                0x15 => Op::LdDtVx { x },
                0x18 => Op::LdStVx { x },
                0x1E => Op::AddIVx { x },
                0x29 => Op::LdFVx { x },
                0x33 => Op::LdBVx { x },
                0x55 => Op::LdIVx { x },
                0x65 => Op::LdVxI { x },
                _ => Op::Invalid,
            },
            _ => Op::Invalid,
        }
    }

    fn execute(&mut self, op: Op) -> Result<(), Box<dyn Error>> {
        match op {
            Op::Cls => {
                for i in 0..self.pixels.len() {
                    self.pixels[i] = false;
                }
            }
            Op::Ret => {
                self.program_counter = self.stack[self.stack_pointer as usize];
                self.stack_pointer -= 1;
            }
            Op::Sys => (),
            Op::Jp { nnn: addr } => {
                self.program_counter = addr;
            }
            Op::Call { nnn: addr } => {
                self.stack_pointer += 1;
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.program_counter = addr;
            }
            Op::Se { x, nn: byte } => {
                let vx = self.registers[x as usize];
                if vx == byte {
                    self.program_counter += 2;
                }
            }
            Op::Sne { x, nn: byte } => {
                let vx = self.registers[x as usize];
                if vx != byte {
                    self.program_counter += 2;
                }
            }
            Op::SeVxVy { x, y } => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                if vx == vy {
                    self.program_counter += 2;
                }
            }
            Op::Ld { x, nn: byte } => self.registers[x as usize] = byte,
            Op::Add { x, nn: byte } => {
                let vx = self.registers[x as usize];
                let (total, _) = vx.overflowing_add(byte);
                // NOTE: This instruction does NOT set the overflow register (vf)
                self.registers[x as usize] = total;
            }
            Op::LdVxVy { x, y } => self.registers[x as usize] = self.registers[y as usize],
            Op::OrVxVy { x, y } => self.registers[x as usize] |= self.registers[y as usize],
            Op::AndVxVy { x, y } => self.registers[x as usize] &= self.registers[y as usize],
            Op::XorVxVy { x, y } => self.registers[x as usize] ^= self.registers[y as usize],
            Op::AddVxVy { x, y } => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                let (total, overflow) = vx.overflowing_add(vy);

                self.registers[0xf] = overflow as u8;
                self.registers[x as usize] = total;
            }
            Op::SubVxVy { x, y } => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                let (total, overflow) = vx.overflowing_sub(vy);
                self.registers[x as usize] = total;
                self.registers[0xf] = !overflow as u8;
            }
            Op::ShrVxVy { x } => {
                let vx = self.registers[x as usize];
                let lsb_is_1 = (vx & 0b00000001).count_ones() == 1;
                self.registers[x as usize] = vx >> 1;
                self.registers[0xf] = if lsb_is_1 { 0x1 } else { 0x0 };
            }
            Op::SubnVxVy { x, y } => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                let (total, overflow) = vy.overflowing_sub(vx);
                self.registers[x as usize] = total;
                self.registers[0xf] = !overflow as u8;
            }
            Op::ShlVxVy { x } => {
                let vx = self.registers[x as usize];
                let msb_is_1 = (vx & 0b10000000).count_ones() == 1;
                self.registers[x as usize] = vx << 1;
                self.registers[0xf] = if msb_is_1 { 0x1 } else { 0x0 };
            }
            Op::SneVxVy { x, y } => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.program_counter += 2;
                }
            }
            Op::LdI { nnn: addr } => {
                self.index_register = addr;
            }
            Op::JpV0 { nnn: addr } => {
                self.program_counter = addr + self.registers[0] as u16;
            }
            Op::Rnd { x, nn: byte } => {
                let mut rng = rand::thread_rng();
                let r = rng.gen::<u8>();
                self.registers[x as usize] = r & byte;
            }
            Op::Drw { x, y, n: nibble } => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                // read nibble bytes from register addrs
                let mut bytes_to_draw: Vec<u8> = vec![];
                for i in 0..nibble {
                    bytes_to_draw.push(self.memory_map[(self.index_register + i as u16) as usize]);
                }

                let mut collision_flag = false;
                let min_row = vy as usize;
                let max_row = vy as usize + bytes_to_draw.len() - 1;
                for row_idx in min_row..=max_row {
                    let b = bytes_to_draw[row_idx - vy as usize];
                    for bit_idx in (0..8).rev() {
                        // TODO: should this wrap around?
                        let pixel_pos = (row_idx * SCREEN_WIDTH + (vx as usize + (7 - bit_idx)))
                            % self.pixels.len();
                        let old_value = self.pixels[pixel_pos];
                        let new_value = (b & 0x1 << bit_idx) > 0;
                        if old_value && new_value {
                            collision_flag = true;
                        }
                        self.pixels[pixel_pos] = old_value ^ new_value;
                    }
                }

                if collision_flag {
                    // TODO: When does the overflow flag get set to false? Should I set to false if there's no overflow?
                    self.registers[0xf] = 0x1; // true
                } else {
                    self.registers[0xf] = 0x0; // false
                }
            }
            Op::Skp { x } => {
                let is_key_pressed = self.keys[self.registers[x as usize] as usize];
                if is_key_pressed {
                    self.program_counter += 2;
                }
            }
            Op::Sknp { x } => {
                // skip if key not pressed
                let is_key_pressed = self.keys[self.registers[x as usize] as usize];
                if !is_key_pressed {
                    self.program_counter += 2;
                }
            }
            Op::LdVxDt { x } => self.registers[x as usize] = self.delay_timer,
            Op::LdVxK { x } => {
                if let Some(found) = self.keys.iter().position(|x| *x) {
                    self.registers[x as usize] = found as u8;
                } else {
                    self.program_counter -= 2;
                }
            }
            Op::LdDtVx { x } => self.delay_timer = self.registers[x as usize],
            Op::LdStVx { x } => self.sound_timer = self.registers[x as usize],
            Op::AddIVx { x } => self.index_register += self.registers[x as usize] as u16,
            Op::LdFVx { x } => {
                self.index_register = FONT_START as u16 + self.registers[x as usize] as u16;
            }
            Op::LdBVx { x } => {
                let vx = self.registers[x as usize];
                self.memory_map[self.index_register as usize] = (vx / 100) % 10;
                self.memory_map[self.index_register as usize + 1] = (vx / 10) % 10;
                self.memory_map[self.index_register as usize + 2] = vx % 10;
            }
            Op::LdIVx { x } => {
                for idx in 0..=x {
                    self.memory_map[(self.index_register + idx as u16) as usize] =
                        self.registers[idx as usize];
                }
                self.index_register = self.index_register + x as u16 + 1;
            }
            Op::LdVxI { x } => {
                for idx in 0..=x {
                    self.registers[idx as usize] =
                        self.memory_map[(self.index_register + idx as u16) as usize];
                }
                self.index_register = self.index_register + x as u16 + 1;
            }
            Op::Invalid => todo!("this will aways fail"),
        }

        Ok(())
    }
}
