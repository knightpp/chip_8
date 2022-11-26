use std::{
    io::{stdout, Stdout, Write},
    sync::{atomic, Arc},
};

use crossterm::{
    cursor,
    terminal::{self, Clear, ClearType},
    ExecutableCommand,
};

use crate::{Chip8, Engine, SimpleRng};

use super::PixelBuf;

pub struct CliEngine {
    pbuf: PixelBuf,
    stdout: Stdout,
    rng: SimpleRng,
}

impl CliEngine {
    pub fn new(stdout: Stdout) -> Self {
        CliEngine {
            pbuf: PixelBuf::new(),
            stdout,
            rng: SimpleRng::new(),
        }
    }
}

impl Engine for CliEngine {
    fn start_loop(&mut self, emulator: &mut Chip8) {
        let exit = Arc::new(atomic::AtomicBool::new(false));
        let r = exit.clone();

        ctrlc::set_handler(move || {
            r.store(true, atomic::Ordering::SeqCst);
        })
        .unwrap();

        let mut stdout = stdout();
        stdout
            .execute(Clear(ClearType::All))
            .unwrap()
            .execute(terminal::SetSize(64, 32))
            .unwrap()
            .execute(cursor::MoveTo(0, 0))
            .unwrap();

        let cpu_sleep = std::time::Duration::from_secs(1) / 600;
        let timers_sleep = std::time::Duration::from_secs(1) / 60;
        let cpu_iterations_before_timers = timers_sleep.as_nanos() / cpu_sleep.as_nanos();

        while !exit.load(atomic::Ordering::SeqCst) {
            for _ in 0..cpu_iterations_before_timers {
                emulator.emulate_cycle(self);
                std::thread::sleep(cpu_sleep);
            }

            emulator.decrement_timers();
        }
    }

    fn clear_screen(&mut self) {
        self.stdout.execute(Clear(ClearType::All)).unwrap();
        self.pbuf.clear();
    }

    fn draw_sprite(&mut self, x: u8, y: u8, height: u8, sprite: &[u8]) -> bool {
        let flipped = self.pbuf.draw_sprite(x, y, height, sprite);

        self.stdout
            // .execute(Clear(ClearType::All))
            // .unwrap()
            .execute(cursor::MoveTo(0, 0))
            .unwrap();
        for row in self.pbuf.gfx {
            for pixel in row {
                if pixel {
                    write!(self.stdout, "#").unwrap();
                } else {
                    write!(self.stdout, " ").unwrap();
                }
            }
            writeln!(self.stdout).unwrap();
        }

        self.stdout.flush().unwrap();

        flipped
    }

    fn rand(&mut self) -> u8 {
        self.rng.next()
    }
}
