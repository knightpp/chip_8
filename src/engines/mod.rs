mod cli;
pub use cli::CliEngine;

mod minifb;
pub use crate::engines::minifb::MinifbEngine;

struct PixelBuf {
    gfx: [[bool; 64]; 32],
}

impl PixelBuf {
    fn new() -> Self {
        Self {
            gfx: [[false; 64]; 32],
        }
    }

    fn clear(&mut self) {
        for row in self.gfx.iter_mut() {
            for pixel in row {
                *pixel = false;
            }
        }
    }

    fn draw_sprite(&mut self, x: u8, y: u8, height: u8, sprite: &[u8]) -> bool {
        let mut flipped = false;
        for yline in 0..height as usize {
            let pixels = sprite[yline as usize];
            for xline in 0..8 {
                let pixel = pixels & (0b1000_0000 >> xline) != 0;
                if pixel {
                    let vbuf_pixel =
                        &mut self.gfx[(y as usize + yline) % 32][(x as usize + xline) % 64];
                    if !flipped && (*vbuf_pixel == true) {
                        flipped = true;
                    }

                    *vbuf_pixel ^= true;
                }
            }
        }

        flipped
    }
}
