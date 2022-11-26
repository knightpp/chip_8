use crate::Chip8;

pub trait Engine {
    fn start_loop(&mut self, emulator: &mut Chip8);

    fn draw_sprite(&mut self, x: u8, y: u8, height: u8, sprite: &[u8]) -> bool;

    fn rand(&mut self) -> u8 {
        unsafe { RNG_STATE.next() }
    }

    fn clear_screen(&mut self);
}

static mut RNG_STATE: RngState = RngState::new();
struct RngState {
    x: u8,
    y: u8,
    z: u8,
    a: u8,
}
impl RngState {
    const fn new() -> Self {
        RngState {
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
