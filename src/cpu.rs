use std::fs::File;
use std::io::prelude::*;
use rand::thread_rng;
use rand::Rng;

#[derive(Debug)]
pub struct Cpu {
    memory: [u8; 0xFFF],
    graphics: [[u8; 64]; 32],
    v: [u8; 16],
    index: u16,
    pc: u16,
    stack: Vec<u16>,
    sound_timer: u8,
    delay_timer: u8,
    current_op: u16,
    key_pressed: [i32; 16],
}

impl Cpu {
    pub const FONT: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0,
                                0x20, 0x60, 0x20, 0x20, 0x70,
                                0xF0, 0x10, 0xF0, 0x80, 0xF0,
                                0xF0, 0x10, 0xF0, 0x10, 0xF0,
                                0x90, 0x90, 0xF0, 0x10, 0x10,
                                0xF0, 0x80, 0xF0, 0x10, 0xF0,
                                0xF0, 0x80, 0xF0, 0x90, 0xF0,
                                0xF0, 0x10, 0x20, 0x40, 0x40,
                                0xF0, 0x90, 0xF0, 0x90, 0xF0,
                                0xF0, 0x90, 0xF0, 0x10, 0xF0,
                                0xF0, 0x90, 0xF0, 0x90, 0x90,
                                0xE0, 0x90, 0xE0, 0x90, 0xE0,
                                0xF0, 0x80, 0x80, 0x80, 0xF0,
                                0xE0, 0x90, 0x90, 0x90, 0xE0,
                                0xF0, 0x80, 0xF0, 0x80, 0xF0,
                                0xF0, 0x80, 0xF0, 0x80, 0x80 ];
    // Public functions
    pub fn new() -> Cpu {
        Cpu {
            memory: [0; 0xFFF],
            graphics: [[0;64];32],
            v: [0; 16],
            index: 0,
            pc: 0x200,
            stack: Vec::<u16>::new(),
            sound_timer: 0,
            delay_timer: 0,
            current_op: 0,
            key_pressed: [0; 16],
        }
    }
    pub fn reset(&mut self) {
        self.memory = [0; 0xFFF];
        self.graphics = [[0;64];32];
        self.v = [0;16];
        self.index = 0;
        self.pc = 0x200;
        self.stack = Vec::<u16>::new();
        self.sound_timer = 0;
        self.delay_timer = 0;
        self.current_op = 0;
        self.key_pressed = [0; 16];

        self.memory[0..80].copy_from_slice(&Cpu::FONT);
    }

    pub fn decrement_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.play_sound();
        }
    }

    #[allow(clippy::unused_io_amount)]
    pub fn load_rom(&mut self, filename: &str) -> Result<(), std::io::Error> {
        self.reset();
        let mut file = File::open(filename)?;
        let filelen = file.metadata().unwrap().len();
        if filelen > 0xFFF - 0x200 {
            panic!("ROM file is greater than 3.5K bytes! Exiting.");
        }
        file.read( &mut self.memory[0x200..0xFFF])?;
        
        Ok(())
    }

    pub fn press_button(&mut self, key: usize) {
        self.key_pressed[key] = 1;
    }

    pub fn release_button(&mut self, key: usize) {
        self.key_pressed[key] = 0;
    }

    // Private functions
    fn play_sound(&mut self) {
        println!("BEEP!");
    }

    #[cfg(not(tarpaulin_include))]
    fn rand() -> u8 {
        thread_rng().gen::<u8>()
    }

    fn get_next_opcode(&mut self) -> u16 {
        let pc = self.pc as usize;
        let command: u16 = (u16::from(self.memory[pc]) << 8) 
                          | u16::from(self.memory[pc+1]);
        self.pc += 2;
        command
    }

    fn clear_screen(&mut self) {
        println!("Clearing screen...");
        self.graphics = [[0;64];32];
    }
}

// Tests
#[cfg(test)]
mod cpu_tests {
    use super::*;

    #[test]
    fn test_new_cpu() {
        let cpu = Cpu::new();
        assert_eq!([0;80], cpu.memory[0..=79]);
        assert_eq!([[0;64];32], cpu.graphics);
        assert_eq!([0;16],cpu.v);
        assert_eq!(0x200, cpu.pc);
        assert!(cpu.stack.is_empty());
        assert_eq!(0, cpu.sound_timer);
        assert_eq!(0, cpu.delay_timer);
        assert_eq!(0, cpu.current_op);
        assert_eq!([0;16], cpu.key_pressed);
    }

    #[test]
    fn test_reset() {
        let mut cpu = Cpu::new();
        cpu.reset();
        assert_eq!(Cpu::FONT, cpu.memory[0..=79]);
        assert_eq!([[0;64];32], cpu.graphics);
        assert_eq!([0;16],cpu.v);
        assert_eq!(0x200, cpu.pc);
        assert!( cpu.stack.is_empty());
        assert_eq!(0, cpu.sound_timer);
        assert_eq!(0, cpu.delay_timer);
        assert_eq!(0, cpu.current_op);
        assert_eq!([0;16], cpu.key_pressed);
    }
    
    #[test]
    fn test_decrement_timers() {
        let mut cpu = Cpu::new();
        cpu.delay_timer = 2;
        cpu.sound_timer = 4;
        cpu.decrement_timers();
        assert_eq!(1, cpu.delay_timer);
        assert_eq!(3, cpu.sound_timer);
    }

    #[test]
    fn test_load_rom() {
        let base_path = env!("CARGO_MANIFEST_DIR");
        let fpath = format!("{}/{}", base_path, "test_opcode.ch8");
        let mut cpu = Cpu::new();
        assert!(cpu.load_rom(&fpath.to_string()).is_ok());
        let fpath = "asdf"; // non-existant file
        assert!(cpu.load_rom(fpath).is_err());
        
    }

    #[test]
    #[should_panic]
    fn test_load_rom_bigfile() {
        let base_path = env!("CARGO_MANIFEST_DIR");
        let fpath = format!("{}/{}", base_path, "big_file.ch8");
        let mut cpu = Cpu::new();
        assert!(cpu.load_rom(&fpath.to_string()).is_err()); //this line panics
    }

    #[test]
    fn test_button_presses() {
        let mut cpu = Cpu::new();
        let button1: usize = 2;
        let button2: usize = 1;
        cpu.press_button(button1);
        cpu.press_button(button2);
        assert_eq!(cpu.key_pressed[button1], 1);
        cpu.release_button(button1);
        assert_eq!(cpu.key_pressed[button1], 0);
        assert_eq!(cpu.key_pressed[button2], 1);
    }

    #[test]
    fn test_get_next_opcode() {
        let mut cpu = Cpu::new();
        cpu.reset();
        cpu.memory[0x200] = 0x12;
        cpu.memory[0x201] = 0x4e;
        let next_op = cpu.get_next_opcode();
        assert_eq!(cpu.pc, 0x202);
        assert!(next_op == 4_686);
    }

    #[test]
    fn test_clear_screen() {
        let mut cpu = Cpu::new();
        // Set some values in graphics mem
        cpu.graphics[0][1] = 4;
        cpu.graphics[2][5] = 5;
        assert_ne!(cpu.graphics, [[0;64];32]);
        cpu.clear_screen();
        assert_eq!(cpu.graphics, [[0;64];32]);
    }
}
