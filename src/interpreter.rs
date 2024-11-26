use std::{error::Error, fs::File, io::Read};

use rand::Rng;

use crate::font::FONT;

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
    SE_VX_VY { x: u4, y: u4 },
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

        let mut buffer = [0 as u8; 4096 - 512];
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

    fn print_program(&self) {
        for i in (PROGRAM_START..PROGRAM_START + self._program_size).step_by(2) {
            let inst = self.fetch_instruction_at(i);
            log::debug!("addr={}  inst={:#06x}  op={:?}", i, inst, self.decode(inst));
        }
        log::debug!("Program Size = {}", self._program_size);
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
        log::debug!(
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
                Op::SE_VX_VY {
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

    fn execute(&mut self, op: Op) -> Result<(), Box<dyn Error>> {
        match op {
            Op::CLS => {
                for i in 0..self.pixels.len() {
                    self.pixels[i] = false;
                }
            }
            Op::RET => {
                self.program_counter = self.stack[self.stack_pointer as usize];
                self.stack_pointer -= 1;
            }
            Op::SYS { addr: _ } => (),
            Op::JP { addr } => {
                log::debug!("jump to addr: {:#05x}", addr);
                self.program_counter = addr;
            }
            Op::CALL { addr } => {
                self.stack_pointer += 1;
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.program_counter = addr;
            }
            Op::SE { x, byte } => {
                let vx = self.registers[x as usize];
                if vx == byte {
                    self.program_counter += 2;
                }
            }
            Op::SNE { x, byte } => {
                let vx = self.registers[x as usize];
                if vx != byte {
                    self.program_counter += 2;
                }
            }
            Op::SE_VX_VY { x, y } => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                if vx == vy {
                    self.program_counter += 2;
                }
            }
            Op::LD { x, byte } => self.registers[x as usize] = byte,
            Op::ADD { x, byte } => {
                let vx = self.registers[x as usize];
                let (total, _) = vx.overflowing_add(byte);
                // NOTE: This instruction does NOT set the overflow register (vf)
                self.registers[x as usize] = total;
            }
            Op::LD_VX_VY { x, y } => self.registers[x as usize] = self.registers[y as usize],
            Op::OR_VX_VY { x, y } => {
                self.registers[x as usize] = self.registers[x as usize] | self.registers[y as usize]
            }
            Op::AND_VX_VY { x, y } => {
                self.registers[x as usize] = self.registers[x as usize] & self.registers[y as usize]
            }
            Op::XOR_VX_VY { x, y } => {
                self.registers[x as usize] = self.registers[x as usize] ^ self.registers[y as usize]
            }
            Op::ADD_VX_VY { x, y } => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];
                let (total, overflow) = vx.overflowing_add(vy);

                self.registers[0xf] = overflow as u8;
                self.registers[x as usize] = total;
            }
            Op::SUB_VX_VY { x, y } => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                let (total, overflow) = vx.overflowing_sub(vy);
                self.registers[x as usize] = total;
                self.registers[0xf] = !overflow as u8;
            }
            Op::SHR_VX_VY { x, y: _ } => {
                let vx = self.registers[x as usize];
                let lsb_is_1 = (vx & 0b00000001).count_ones() == 1;
                self.registers[x as usize] = vx >> 1;
                self.registers[0xf] = if lsb_is_1 { 0x1 } else { 0x0 };
            }
            Op::SUBN_VX_VY { x, y } => {
                let vx = self.registers[x as usize];
                let vy = self.registers[y as usize];

                let (total, overflow) = vy.overflowing_sub(vx);
                self.registers[x as usize] = total;
                self.registers[0xf] = !overflow as u8;
            }
            Op::SHL_VX_VY { x, y: _ } => {
                let vx = self.registers[x as usize];
                let msb_is_1 = (vx & 0b10000000).count_ones() == 1;
                self.registers[x as usize] = vx << 1;
                self.registers[0xf] = if msb_is_1 { 0x1 } else { 0x0 };
            }
            Op::SNE_VX_VY { x, y } => {
                if self.registers[x as usize] != self.registers[y as usize] {
                    self.program_counter += 2;
                }
            }
            Op::LD_I { addr } => {
                self.index_register = addr;
            }
            Op::JP_V0 { addr } => {
                self.program_counter = addr + self.registers[0] as u16;
            }
            Op::RND { x, byte } => {
                let mut rng = rand::thread_rng();
                let r = rng.gen::<u8>();
                self.registers[x as usize] = r & byte;
            }
            Op::DRW { x, y, nibble } => {
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
            Op::LD_VX_DT { x } => self.registers[x as usize] = self.delay_timer,
            Op::LD_VX_K { x } => {
                if let Some(found) = self.keys.iter().position(|x| *x == true) {
                    self.registers[x as usize] = found as u8;
                } else {
                    self.program_counter -= 2;
                }
            }
            Op::LD_DT_VX { x } => self.delay_timer = self.registers[x as usize],
            Op::LD_ST_VX { x } => self.sound_timer = self.registers[x as usize],
            Op::ADD_I_VX { x } => {
                self.index_register = self.registers[x as usize] as u16 + self.index_register
            }
            Op::LD_F_VX { x } => {
                self.index_register = FONT_START as u16 + self.registers[x as usize] as u16;
            }
            Op::LD_B_VX { x } => {
                let vx = self.registers[x as usize];
                self.memory_map[self.index_register as usize] = (vx / 100) % 10;
                self.memory_map[self.index_register as usize + 1] = (vx / 10) % 10;
                self.memory_map[self.index_register as usize + 2] = vx % 10;
            }
            Op::LD_I_VX { x } => {
                for idx in 0..=x {
                    self.memory_map[(self.index_register + idx as u16) as usize] =
                        self.registers[idx as usize];
                }
                self.index_register = self.index_register + x as u16 + 1;
            }
            Op::LD_VX_I { x } => {
                for idx in 0..=x {
                    self.registers[idx as usize] =
                        self.memory_map[(self.index_register + idx as u16) as usize];
                }
                self.index_register = self.index_register + x as u16 + 1;
            }
            Op::INVALID => todo!("this will aways fail"),
        }

        Ok(())
    }
}
