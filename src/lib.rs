mod disassembler;
pub use disassembler::disassemble_file;
mod chip8;
pub mod engines;
pub use chip8::Chip8;

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

fn word_to_nibbles([hi, lo]: &[u8; 2]) -> [u8; 4] {
    [
        (hi & 0b1111_0000) >> 4,
        hi & 0b0000_1111,
        (lo & 0b1111_0000) >> 4,
        lo & 0b0000_1111,
    ]
}

pub trait Engine {
    fn start_loop(&mut self, emulator: &mut Chip8);

    fn draw_sprite(&mut self, x: u8, y: u8, height: u8, sprite: &[u8]) -> bool;

    fn rand(&mut self) -> u8;

    fn clear_screen(&mut self);
}

struct SimpleRng {
    x: u8,
    y: u8,
    z: u8,
    a: u8,
}

impl SimpleRng {
    const fn new() -> Self {
        SimpleRng {
            x: 0,
            y: 0,
            z: 0,
            a: 1,
        }
    }
    fn next(&mut self) -> u8 {
        let t = self.x ^ (self.x << 4);
        self.x = self.y;
        self.y = self.z;
        self.z = self.a;
        self.a = self.z ^ t ^ (self.z >> 1) ^ (t << 1);
        self.a
    }
}

pub(crate) trait MergeNibbles {
    type Output;
    /// big-endian aware
    fn merge_nibbles(&self) -> Self::Output;
}

impl MergeNibbles for [u8; 3] {
    type Output = u16;
    fn merge_nibbles(&self) -> Self::Output {
        ((self[0] as u16) << 8) | ((self[1] as u16) << 4) | (self[2] as u16)
    }
}

impl MergeNibbles for [u8; 2] {
    type Output = u8;

    fn merge_nibbles(&self) -> Self::Output {
        ((self[0] as u8) << 4) | (self[1] as u8)
    }
}
