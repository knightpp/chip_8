mod disassembler;
pub use disassembler::disassemble_file;
mod traits;
pub use traits::Engine;
use traits::*;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Debug, Default)]
struct Registers {
    v: [u8; 16],
    i: u16,
    sp: u16,
    pc: u16,
}

impl Registers {
    fn new() -> Registers {
        Registers {
            pc: 0x0200,
            ..Default::default()
        }
    }
}

pub struct Chip8 {
    mem: [u8; 4096],
    regs: Registers,
    stack: [u16; 32],
    key_state: [bool; 16],
    delay_timer: u8,
    sound_timer: u8,
    engine: Box<dyn Engine>,
}

impl Chip8 {
    pub fn new(engine: Box<dyn Engine>) -> Chip8 {
        let mut mem = [0; 4096];
        mem[..80].copy_from_slice(&FONTSET);

        Chip8 {
            mem: mem,
            stack: [0; 32],
            regs: Registers::new(),
            delay_timer: 0,
            sound_timer: 0,
            engine,
            key_state: [false; 16],
        }
    }

    pub fn load_game(&mut self, data: &[u8]) {
        self.mem[0x200..0x200 + data.len()].copy_from_slice(data);
    }

    fn skip_next_instruction(&mut self) {
        self.regs.pc += 2;
    }

    fn jump(&mut self, address: u16) {
        self.regs.pc = address;
    }

    fn stack_push(&mut self, value: u16) {
        self.stack[self.regs.sp as usize] = value;
        self.regs.sp += 1;
    }

    fn stack_pop(&mut self) -> u16 {
        self.regs.sp -= 1;
        self.stack[self.regs.sp as usize]
    }

    pub fn emulate_cycle(&mut self) {
        macro_rules! v {
            ($name:tt) => {
                self.regs.v[($name) as usize]
            };
        }

        let instruction: [u8; 2] = [
            self.mem[self.regs.pc as usize],
            self.mem[self.regs.pc as usize + 1],
        ];
        match word_to_nibbles(&instruction) {
            // 00E0
            [0, 0, 0xE, 0] => {
                self.engine.clear_screen();
            }
            // 00EE
            [0, 0, 0xE, 0xE] => {
                let address = self.stack_pop();
                self.jump(address);
            }
            // 0NNN Call
            [0, nnn @ ..] => {
                self.stack_push(self.regs.pc);
                self.jump(nnn.merge_nibbles());
            }
            // 1NNN
            [1, nnn @ ..] => {
                self.jump(nnn.merge_nibbles());
            }
            // 2NNN
            [0x2, nnn @ ..] => {
                let mem_loc = nnn.merge_nibbles() as usize;
                let address = ((self.mem[mem_loc] as u16) << 8) | (self.mem[mem_loc + 1] as u16);
                self.stack_push(address);
                self.jump(nnn.merge_nibbles());
            }
            // 3XNN
            [0x3, x, nn @ ..] => {
                if v!(x) == nn.merge_nibbles() {
                    self.skip_next_instruction();
                }
            }
            // 4XNN
            [0x4, x, nn @ ..] => {
                if v!(x) != nn.merge_nibbles() {
                    self.skip_next_instruction();
                }
            }
            // 5XY0
            [0x5, x, y, 0] => {
                if v!(x) == v!(y) {
                    self.skip_next_instruction();
                }
            }
            // 6XNN
            [0x6, x, nn @ ..] => {
                v![x] = nn.merge_nibbles();
            }
            // 7XNN
            [0x7, x, nn @ ..] => {
                v![x] += nn.merge_nibbles();
            }
            // 8XY0
            [0x8, x, y, 0] => {
                v![x] = v![y];
            }
            // 8XY1
            [0x8, x, y, 1] => v![x] |= v![y],
            // 8XY2
            [0x8, x, y, 0x2] => v![x] &= v![y],
            // 8XY3
            [0x8, x, y, 0x3] => v![x] ^= v![y],
            // 8XY4
            [0x8, x, y, 0x4] => {
                let res = v![x] as u16 + v![y] as u16;
                v![x] = res as u8;
                v![0xF] = ((res >> 8) > 0) as u8;
            }
            // 8XY5
            [0x8, x, y, 0x5] => {
                // TODO: borrow flag
                let borrow = v![x] < v![y];
                self.regs.v[0xF] = !borrow as u8;
                v![x] -= v![y];
            }
            // 8XY6
            [0x8, x, _y, 0x6] => {
                let lsb = v![x] & 0b0000_0001;
                v![0xF] = lsb;
                v![x] >>= 1;
            }
            // 8XY7
            [0x8, x, y, 0x7] => {
                // TODO: borrow flag
                let borrow = v![x] < v![y];
                v![0xF] = !borrow as u8;
                v![x] -= v![y];
            }
            // 8XYE
            [0x8, x, _y, _e] => {
                let msb = v![x] & 0b1000_0000;
                v![0xF] = msb;
                v![x] <<= 1;
            }
            // 9XY0
            [0x9, x, y, 0] => {
                if v![x] != v![y] {
                    self.skip_next_instruction();
                }
            }
            // ANNN
            [0xA, nnn @ ..] => {
                self.regs.i = nnn.merge_nibbles();
            }
            // BNNN
            [0xB, nnn @ ..] => {
                self.jump(v![0] as u16 + nnn.merge_nibbles());
            }
            // CXNN
            [0xC, x, nn @ ..] => {
                v![x] = self.engine.rand() & nn.merge_nibbles();
            }
            // DXYN
            [0xD, x, y, n] => {
                let i = self.regs.i as usize;
                let flipped =
                    self.engine
                        .draw_sprite(v![x], v![y], n, &self.mem[i..i + (n as usize) * 8]);
                v![0xF] = flipped as u8;
            }
            // EX9E
            [0xE, x, 0x9, 0xE] => {
                if self.key_state[v![x] as usize] {
                    self.skip_next_instruction();
                }
            }
            // EXA1
            [0xE, x, 0xA, 0x1] => {
                if !self.key_state[v![x] as usize] {
                    self.skip_next_instruction();
                }
            }
            // FX07
            [0xF, x, 0, 0x7] => {
                v![x] = self.delay_timer;
            }
            // FX0A
            [0xF, x, 0, 0xA] => todo!(),
            // FX15
            [0xF, x, 1, 0x5] => self.delay_timer = v![x],
            // FX18
            [0xF, x, 1, 0x8] => self.sound_timer = v![x],
            // FX1E
            [0xF, x, 1, 0xE] => {
                self.regs.i += v![x] as u16;
            }
            // FX29
            [0xF, x, 0x2, 0x9] => self.regs.i = v![x] as u16 * 5,
            // FX33
            [0xF, x, 0x3, 0x3] => {
                let i = self.regs.i as usize;
                self.mem[i + 0] = v![x] / 100;
                self.mem[i + 1] = (v![x] / 10) % 10;
                self.mem[i + 2] = (v![x] % 100) % 10;
            }
            // FX55
            [0xF, x, 0x5, 0x5] => {
                let offset = self.regs.i as usize;
                for (i, v) in self.regs.v[..=x as usize].iter().enumerate() {
                    self.mem[offset + i] = *v;
                }
            }
            // FX65
            [0xF, x, 0x6, 0x5] => {
                let offset = self.regs.i as usize;
                for (i, v) in self.regs.v[..=x as usize].iter_mut().enumerate() {
                    *v = self.mem[offset + i];
                }
            }
            _ => {
                panic!("unknown instruction {:#02X?}", instruction);
            }
        }
        self.regs.pc += 2;
    }
}

/**
two bytes
```
|   b1    | |   b2    |
|----|----| |----|----|
| n1 | n2 | | n3 | n4 |
```
**/
fn word_to_nibbles([hi, lo]: &[u8; 2]) -> [u8; 4] {
    let nibbles = [
        (hi & 0b1111_0000) >> 4,
        hi & 0b0000_1111,
        (lo & 0b1111_0000) >> 4,
        lo & 0b0000_1111,
    ];

    nibbles
}