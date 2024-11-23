use std::{
    fs::File,
    io::{BufReader, Error, Read},
    path::Path,
};

// wrap u8 for now
type u4 = u8;

// wrap u16 for now
type u12 = u16;

enum Op {
    NYI, // Not yet implemented. For use during development of Chip8 Interpreter

    CLS,
    RET,
    SYS { addr: u12 },
    JP { addr: u12 },
    CALL { addr: u12 },
    SE { vx: u4, byte: u8 },
    SNE { vx: u4, byte: u8 },
    SN2 { vx: u4, vy: u4 },
}

const MEMORY_SIZE: usize = 4096;

struct Interpreter {
    memory_map: [u8; MEMORY_SIZE],
    stack: [u16; 16],

    // registers
    registers_vx: [u8; 16], // also called Vx
    register_i: u16,        // usually only stores lowest 12 bits, for memory addresses
    // pseudo-registers
    program_counter: u16,
    stack_pointer: u8,
}

const PROGRAM_START: u16 = 256;

impl Interpreter {
    fn new() -> Self {
        // initialize memory map
        Interpreter {
            memory_map: [0; 4096],
            stack: [0; 16],

            registers_vx: [0; 16],
            register_i: 0,

            program_counter: PROGRAM_START,
            stack_pointer: 0,
        }
    }

    /// Reads a program from a file and writes it into the memory_map
    fn read_program_from_file(&mut self, p: &str) -> Result<(), Error> {
        let mut file = File::open(p)?;

        let mut buffer = [0 as u8; 4096 - 512];
        file.read(&mut buffer)?;
        for (idx, b) in buffer.iter().enumerate() {
            self.memory_map[PROGRAM_START as usize + idx] = *b;
        }

        Ok(())
    }

    fn fetch(&mut self) -> u16 {
        // // gets the next two bytes and sets the program counter forward
        let first = self.memory_map[self.program_counter as usize];
        let second = self.memory_map[(self.program_counter + 1) as usize];
        let instruction = ((first as u16) << 8) | second as u16;

        self.program_counter += 2;
        instruction
    }

    fn decode(&self, instruction: u16) -> Op {
        let first_nibble: u4 = (instruction >> 12) as u8;
        let twelve_bits: u12 = (0x0111 as u16) & instruction; // TODO: drop the first bit
        match first_nibble {
            0 => match instruction {
                0x00E0 => Op::CLS,
                0x00EE => Op::RET,
                _ => Op::SYS { addr: twelve_bits },
            },
            1 => Op::JP { addr: twelve_bits },
            // 2nnn - CALL addr
            // 3xkk - SE Vx, byte
            // 4xkk - SNE Vx, byte
            // 5xy0 - SE Vx, Vy
            // 6xkk - LD Vx, byte
            // 7xkk - ADD Vx, byte
            // 8xy0 - LD Vx, Vy
            // 8xy1 - OR Vx, Vy
            // 8xy2 - AND Vx, Vy
            // 8xy3 - XOR Vx, Vy
            // 8xy4 - ADD Vx, Vy
            // 8xy5 - SUB Vx, Vy
            // 8xy6 - SHR Vx {, Vy}
            // 8xy7 - SUBN Vx, Vy
            // 8xyE - SHL Vx {, Vy}
            // 9xy0 - SNE Vx, Vy
            // Annn - LD I, addr
            // Bnnn - JP V0, addr
            // Cxkk - RND Vx, byte
            // Dxyn - DRW Vx, Vy, nibble
            // Ex9E - SKP Vx
            // ExA1 - SKNP Vx
            // Fx07 - LD Vx, DT
            // Fx0A - LD Vx, K
            // Fx15 - LD DT, Vx
            // Fx18 - LD ST, Vx
            // Fx1E - ADD I, Vx
            // Fx29 - LD F, Vx
            // Fx33 - LD B, Vx
            // Fx55 - LD [I], Vx
            // Fx65 - LD Vx, [I]
            _ => Op::NYI,
        }
    }

    fn execute(&self, op: Op) -> Result<(), Error> {
        match op {
            Op::CLS => println!("clear screen"),
            _ => (), // TODO:
        }

        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let mut interpreter = Interpreter::new();

    // read program
    interpreter.read_program_from_file("roms/IBM_LOGO.ch8")?;

    // run the program
    while interpreter.program_counter < MEMORY_SIZE as u16 {
        let instruction = interpreter.fetch();
        let op = interpreter.decode(instruction);
        interpreter.execute(op)?;
    }

    println!("done");
    Ok(())
}
