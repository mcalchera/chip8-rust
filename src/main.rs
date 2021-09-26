mod cpu;
use crate::cpu::Cpu;
use std::env;
extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::thread::sleep;
use std::time::Duration;

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
    let black = Color::RGB(0x5a, 0x39, 0x21); // "activated" pixel
    let white = Color::RGB(0xff, 0xff, 0xb5); // "inactive" pixel

    if !cpu.load_rom(config.rom.as_str()).is_ok() {
        println!("chip8: error loading ROM file: \"{}\"", config.rom);
        std::process::exit(1);
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Chip-8 Rust", 640, 320)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(white);
    canvas.clear();
    canvas.present();
    sleep(Duration::from_millis(2000));
}
