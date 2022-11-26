use chip_8::{engines, Engine};
use std::io::stdout;

fn main() {
    start_minifb_engine();
    return;
    start_cli_engine();
}

fn start_minifb_engine() {
    let path = "./roms/pong.rom";

    let mut engine = engines::MinifbEngine::create(40).unwrap();
    let mut chip = chip_8::Chip8::new();

    let data = std::fs::read(path).unwrap();
    chip.load_game(&data);

    engine.start_loop(&mut chip);
}

fn start_cli_engine() {
    let path = "./roms/pong.rom";

    let mut engine = engines::CliEngine::new(stdout());
    let mut chip = chip_8::Chip8::new();

    let data = std::fs::read(path).unwrap();
    chip.load_game(&data);

    engine.start_loop(&mut chip);
}
