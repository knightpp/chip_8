use minifb::{self, Key, Window, WindowOptions};

use crate::{Chip8, Engine};

use super::PixelBuf;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct MinifbEngine {
    pbuf: PixelBuf,
    buffer: Vec<u32>,
    width: usize,
    height: usize,
    scale: usize,
    window: Window,
}

impl MinifbEngine {
    pub fn create(scale: usize) -> Result<Self> {
        let width = 64 * scale;
        let height = 32 * scale;

        let mut window = Window::new(
            "Test - ESC to exit",
            width,
            height,
            WindowOptions::default(),
        )?;

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        Ok(MinifbEngine {
            buffer: vec![0; width * height],
            pbuf: PixelBuf::new(),
            height,
            width,
            scale,
            window,
        })
    }

    fn draw_to_window(&mut self) -> Result<()> {
        for (y, row) in self.pbuf.gfx.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                let color = if *pixel { 0xFF } else { 0x0 };
                for x_offset in 0..self.scale {
                    for y_offset in 0..self.scale {
                        self.buffer[(x * self.scale)
                            + x_offset
                            + (y * self.scale * self.width)
                            + self.width * y_offset] = color;
                    }
                }
            }
        }

        self.window
            .update_with_buffer(&self.buffer, self.width, self.height)?;

        Ok(())
    }
}

impl Engine for MinifbEngine {
    fn start_loop(&mut self, emulator: &mut Chip8) {
        let cpu_sleep = std::time::Duration::from_secs(1) / 600;
        let timers_sleep = std::time::Duration::from_secs(1) / 60;
        let cpu_iterations_before_timers = timers_sleep.as_nanos() / cpu_sleep.as_nanos();

        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            for _ in 0..cpu_iterations_before_timers {
                emulator.emulate_cycle(self);
                std::thread::sleep(cpu_sleep);
            }

            emulator.decrement_timers();
            self.window
                .update_with_buffer(&self.buffer, self.width, self.height)
                .unwrap();
        }
    }

    fn draw_sprite(&mut self, x: u8, y: u8, height: u8, sprite: &[u8]) -> bool {
        let flipped = self.pbuf.draw_sprite(x, y, height, sprite);

        self.draw_to_window().unwrap();

        flipped
    }

    fn clear_screen(&mut self) {
        self.pbuf.clear();
        for el in self.buffer.iter_mut() {
            *el = 0;
        }
    }
}
