mod cpu;
use crate::cpu::Cpu;
use std::env;

struct Config {
    pub rom: String,
}

fn process_args(args: &[String]) -> Config {
    if args.len() != 2 {
        println!("Usage: {} <rom file>", args[0]);
        std::process::exit(1);
    }

    let fname = &args[1];

    Config {
        rom: fname.to_string(),
    }
}

#[cfg(not(tarpaulin_include))]
fn main() {
    let mut cpu = Cpu::new();
    let args: Vec<String> = env::args().collect(); 
    let config = process_args(&args);

    if !cpu.load_rom(config.rom.as_str()).is_ok() {
        println!("chip8: error loading ROM file: \"{}\"", config.rom);
        std::process::exit(1);
    }

}
