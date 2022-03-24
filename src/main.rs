mod cpu;
use crate::cpu::Cpu;
use std::env;
use std::thread::sleep;
use std::time::{Instant, Duration};

extern crate sdl2;
use sdl2::event::Event;
use sdl2::pixels::Color;

pub struct Config {
    pub rom: String,
    pub scale: u32, // How big to make a single pixel
    pub black: Color, // The color of an "activated" pixel
    pub white: Color, // The color of an "inactive" pixel
}

fn process_args(args: &[String]) -> Config {
    if args.len() != 2 {
        println!("Usage: {} <rom file>", args[0]);
        std::process::exit(1);
    }

    let fname = &args[1];

    Config {
        rom: fname.to_string(),
        scale: 10,
        black: Color::RGB(0x5a, 0x39, 0x21),
        white: Color::RGB(0xff, 0xff, 0xb5),
    }
}


#[cfg(not(tarpaulin_include))]
fn main() {
    let mut cpu = Cpu::new();
    let args: Vec<String> = env::args().collect(); 
    let config = process_args(&args);
    let window_height = Cpu::GFX_HEIGHT as u32 * config.scale;
    let window_width = Cpu::GFX_WIDTH as u32 * config.scale;

    if cpu.load_rom(config.rom.as_str()).is_err() {
        println!("chip8: error loading ROM file: \"{}\"", config.rom);
        std::process::exit(1);
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Chip-8 Rust", window_width, window_height)
        .position_centered()
        .build()
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    // Game loop

    let mut start_time = Instant::now();
    const DELTA: Duration = Duration::from_millis(16);
    'gameloop: loop {
        cpu.advance_state();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'gameloop;
                },
                Event::KeyDown {..} |
                Event::KeyUp {..} => {
                    cpu.process_input(event);
                },
                _ => (),
            }
        }
        // TODO: timing
        sleep(Duration::from_millis(2));
        let new_time = Instant::now();

        if new_time.duration_since(start_time) >= DELTA {
            cpu.decrement_timers();
            start_time = Instant::now();
        }
        cpu.update_graphics(&config, &mut canvas);
    }

}
