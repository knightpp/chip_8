use argh::FromArgValue;
use chip_8::{engines, Engine};
use std::{io::stdout, path::PathBuf};

#[derive(argh::FromArgs)]
/// Simple chip8 emulator
struct Args {
    #[argh(option, default = "Mode::Minifb")]
    /// you can choose engine
    mode: Mode,

    #[argh(positional)]
    rom_path: PathBuf,
}

enum Mode {
    Minifb,
    Cli,
}
impl FromArgValue for Mode {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        match value {
            "minifb" => Ok(Mode::Minifb),
            "cli" => Ok(Mode::Cli),
            _ => Err("unknown mode".to_owned()),
        }
    }
}

fn main() {
    let args: Args = argh::from_env();

    match args.mode {
        Mode::Minifb => start_minifb_engine(args.rom_path),
        Mode::Cli => start_cli_engine(args.rom_path),
    }
}

fn start_minifb_engine(path: PathBuf) {
    let mut engine = engines::MinifbEngine::create(40).unwrap();
    let mut chip = chip_8::Chip8::new();

    let data = std::fs::read(path).unwrap();
    chip.load_game(&data);

    engine.start_loop(&mut chip);
}

fn start_cli_engine(path: PathBuf) {
    let mut engine = engines::CliEngine::new(stdout());
    let mut chip = chip_8::Chip8::new();

    let data = std::fs::read(path).unwrap();
    chip.load_game(&data);

    engine.start_loop(&mut chip);
}
