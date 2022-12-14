use std::fmt;

use comfy_table::Table;

use crate::{word_to_nibbles, Engine, MergeNibbles, FONTSET};

#[derive(Debug, Default)]
struct Registers {
    v: [u8; 16],
    i: u16,
    sp: u8,
    pc: u16,
}

impl Registers {
    fn new() -> Registers {
        Registers {
            pc: 0x200,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct Quircks {
    pub load_store: bool,
}

pub struct Chip8 {
    mem: [u8; 4096],
    regs: Registers,
    stack: [u16; 16],
    key_state: [bool; 16],
    delay_timer: u8,
    sound_timer: u8,
    pub quircks: Quircks,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut mem = [0; 4096];
        mem[..80].copy_from_slice(&FONTSET);

        Chip8 {
            mem,
            stack: [0; 16],
            regs: Registers::new(),
            delay_timer: 0,
            sound_timer: 0,
            key_state: [false; 16],
            quircks: Quircks::default(),
        }
    }

    pub fn load_game(&mut self, data: &[u8]) {
        self.mem[0x200..0x200 + data.len()].copy_from_slice(data);
    }

    fn skip_next_instruction(&mut self) {
        self.regs.pc += 2;
    }

    fn jump(&mut self, address: u16) {
        // -2 because we unconditionally subtract every cycle
        self.regs.pc = address - 2;
    }

    fn stack_push(&mut self, value: u16) {
        self.stack[self.regs.sp as usize] = value;
        self.regs.sp += 1;
    }

    fn stack_pop(&mut self) -> u16 {
        self.regs.sp -= 1;
        self.stack[self.regs.sp as usize]
    }

    fn call(&mut self, address: u16) {
        self.stack_push(self.regs.pc + 2);
        self.jump(address);
    }

    pub fn emulate_cycle<T: Engine>(&mut self, engine: &mut T) {
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
                engine.clear_screen();
            }
            // 00EE
            [0, 0, 0xE, 0xE] => {
                let address = self.stack_pop();
                self.jump(address);
            }
            // 0NNN Call
            [0, _nnn @ ..] => {
                todo!();
                // self.call(nnn.merge_nibbles());

                // let mem_loc = nnn.merge_nibbles() as usize;
                // let address = ((self.mem[mem_loc] as u16) << 8) | (self.mem[mem_loc + 1] as u16);
                // self.stack_push(address);
                // self.jump(nnn.merge_nibbles());
            }
            // 1NNN
            [1, nnn @ ..] => {
                self.jump(nnn.merge_nibbles());
            }
            // 2NNN
            [0x2, nnn @ ..] => {
                self.call(nnn.merge_nibbles());
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
                v![x] = v![x].wrapping_add(nn.merge_nibbles());
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
                let sum = v![x] as u16 + v![y] as u16;
                v![x] = sum as u8;
                v![0xF] = (sum > 0xFF) as u8;
            }
            // 8XY5
            [0x8, x, y, 0x5] => {
                let borrow = v![x] < v![y];
                v![0xF] = !borrow as u8;
                v![x] = v![x].wrapping_sub(v![y]);
            }
            // 8XY6
            [0x8, x, y, 0x6] => {
                v![0xF] = v![y] & 0b0000_0001;
                v![x] = v![y] >> 1;
            }
            // 8XY7
            [0x8, x, y, 0x7] => {
                let borrow = v![x] > v![y];
                v![0xF] = !borrow as u8;
                v![x] = v![y] - v![x];
            }
            // 8XYE
            [0x8, x, y, 0xE] => {
                v![0xF] = (v![y] & 0b1000_0000) >> 7;
                v![x] = v![y] << 1;
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
                v![x] = engine.rand() & nn.merge_nibbles();
            }
            // DXYN
            [0xD, x, y, n] => {
                let i = self.regs.i as usize;
                let flipped =
                    engine.draw_sprite(v![x], v![y], n, &self.mem[i..i + (n as usize) * 8]);
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
            [0xF, _x, 0, 0xA] => todo!(),
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
                self.mem[i] = v![x] / 100;
                self.mem[i + 1] = (v![x] % 100) / 10;
                self.mem[i + 2] = v![x] % 10;
            }
            // FX55
            [0xF, x, 0x5, 0x5] => {
                // Store the values of registers V0 to VX inclusive in memory starting at address I
                // I is set to I + X + 1 after operation
                let offset = self.regs.i as usize;
                for (i, v) in self.regs.v[..=x as usize].iter().enumerate() {
                    self.mem[offset + i] = *v;
                }

                if self.quircks.load_store {
                    self.regs.i += x as u16 + 1;
                }
            }
            // FX65
            [0xF, x, 0x6, 0x5] => {
                // Fill registers V0 to VX inclusive with the values stored in memory starting at address I
                // I is set to I + X + 1 after operation
                let offset = self.regs.i as usize;
                for (i, v) in self.regs.v[..=x as usize].iter_mut().enumerate() {
                    *v = self.mem[offset + i];
                }

                if self.quircks.load_store {
                    self.regs.i += x as u16 + 1;
                }
            }
            _ => {
                panic!("unknown instruction {:#02X?}", instruction);
            }
        }

        self.regs.pc += 2;
    }

    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut table = Table::new();
        table
            .set_header(vec![
                "Other registers",
                "V0-V7",
                "V8-VF",
                "Stack 0-7",
                "Stack 8-F",
            ])
            .add_row(vec![
                format!("I 0x{:04X}", self.regs.i),
                format!("V0 0x{:02X}", self.regs.v[0]),
                format!("V8 0x{:02X}", self.regs.v[8]),
                format!("SP0 0x{:04X}", self.stack[0]),
                format!("SP8 0x{:04X}", self.stack[8]),
            ])
            .add_row(vec![
                format!("PC 0x{:04X}", self.regs.pc),
                format!("V1 0x{:02X}", self.regs.v[1]),
                format!("V9 0x{:02X}", self.regs.v[9]),
                format!("SP1 0x{:04X}", self.stack[1]),
                format!("SP9 0x{:04X}", self.stack[9]),
            ])
            .add_row(vec![
                format!("DT 0x{:02X}", self.delay_timer),
                format!("V2 0x{:02X}", self.regs.v[2]),
                format!("VA 0x{:02X}", self.regs.v[0xA]),
                format!("SP2 0x{:04X}", self.stack[2]),
                format!("SPA 0x{:04X}", self.stack[0xA]),
            ])
            .add_row(vec![
                format!("ST 0x{:02X}", self.sound_timer),
                format!("V3 0x{:02X}", self.regs.v[3]),
                format!("VB 0x{:02X}", self.regs.v[0xB]),
                format!("SP3 0x{:04X}", self.stack[3]),
                format!("SPB 0x{:04X}", self.stack[0xB]),
            ])
            .add_row(vec![
                format!("SP 0x{:04X}", self.regs.sp),
                format!("V4 0x{:02X}", self.regs.v[4]),
                format!("VC 0x{:02X}", self.regs.v[0xC]),
                format!("SP4 0x{:04X}", self.stack[4]),
                format!("SPC 0x{:04X}", self.stack[0xC]),
            ])
            .add_row(vec![
                String::new(),
                format!("V5 0x{:02X}", self.regs.v[5]),
                format!("VD 0x{:02X}", self.regs.v[0xD]),
                format!("SP5 0x{:04X}", self.stack[5]),
                format!("SPD 0x{:04X}", self.stack[0xD]),
            ])
            .add_row(vec![
                String::new(),
                format!("V6 0x{:02X}", self.regs.v[6]),
                format!("VE 0x{:02X}", self.regs.v[0xE]),
                format!("SP6 0x{:04X}", self.stack[6]),
                format!("SPE 0x{:04X}", self.stack[0xE]),
            ])
            .add_row(vec![
                String::new(),
                format!("V7 0x{:02X}", self.regs.v[7]),
                format!("VF 0x{:02X}", self.regs.v[0xF]),
                format!("SP7 0x{:04X}", self.stack[7]),
                format!("SPF 0x{:04X}", self.stack[0xF]),
            ]);

        f.write_fmt(format_args!("{}", table))
    }
}
