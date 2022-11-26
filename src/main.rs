use crossterm::{
    cursor,
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use std::io::{stdout, Stdout, Write};

use chip_8::Engine;

struct CliEngine {
    vbuf: [bool; 32 * 64],
    stdout: Stdout,
}

impl CliEngine {
    pub fn new(stdout: Stdout) -> Self {
        CliEngine {
            vbuf: [false; 32 * 64],
            stdout,
        }
    }
}

impl Engine for CliEngine {
    fn process_key_press(&self) {
        todo!()
    }

    fn clear_screen(&mut self) {
        self.stdout.execute(Clear(ClearType::All)).unwrap();
        self.vbuf.iter_mut().for_each(|e| *e = false);
    }

    fn draw_sprite(&mut self, x: u8, y: u8, height: u8, sprite: &[u8]) -> bool {
        let vbuf = &mut self.vbuf[x as usize * y as usize + x as usize..];
        let mut flipped = false;
        for yline in 0..height as usize {
            let pixel = sprite[yline as usize];
            for xline in 0..8 {
                if pixel & (0x80 >> xline) != 0 {
                    let vbuf_index = x as usize + xline + (y as usize + yline) * 64;
                    if !flipped && (vbuf[vbuf_index] == true) {
                        flipped = true;
                    }

                    vbuf[vbuf_index] ^= true;
                }
            }
        }

        self.stdout
            .execute(Clear(ClearType::All))
            .unwrap()
            .execute(cursor::MoveTo(0, 0))
            .unwrap();
        for row in self.vbuf.chunks_exact(32) {
            for pixel in row {
                if *pixel {
                    // ❏
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
}

fn main() {
    let path = "./roms/pong.rom";
    // println!(
    //     "{}",
    //     chip_8::disassemble_file(std::fs::File::open(path).unwrap())
    // );
    // return;
    let data = std::fs::read(path).unwrap();

    let mut stdout = stdout();
    stdout
        .execute(Clear(ClearType::All))
        .unwrap()
        // .execute(cursor::Hide)
        // .unwrap()
        .execute(cursor::MoveTo(0, 0))
        .unwrap();

    let engine = CliEngine::new(stdout);
    let mut chip = chip_8::Chip8::new(Box::new(engine));

    chip.load_game(&data);

    loop {
        chip.emulate_cycle();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}
