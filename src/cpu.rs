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
    current_op: (u8,u8,u8,u8),
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
            current_op: (0,0,0,0),
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
        self.current_op = (0,0,0,0);
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

    fn get_next_opcode(&mut self) -> (u8,u8,u8,u8) {
        let pc = self.pc as usize;
        let command: u16 = (u16::from(self.memory[pc]) << 8) 
                          | u16::from(self.memory[pc+1]);
        self.pc += 2;
        let op = (
            (command >> 12) as u8 & 0xF,
            (command >>  8) as u8 & 0xF,
            (command >>  4) as u8 & 0xF,
            command         as u8 & 0xF);
        op
    }

    fn clear_screen(&mut self) {
        println!("Clearing screen...");
        self.graphics = [[0;64];32];
    }

    fn execute_op(&mut self) {
        self.current_op = self.get_next_opcode();
        match self.current_op.0 {
            0x0 => match self.current_op.1 {
                0x0 => match self.current_op.2 {
                    0xE0 => self.clear_screen(),
                    0xEE => self.op_00ee(),
                    _ => self.unimplemented(),
                }
                _ => self.unimplemented(), // We don't support 0x0NNN instructions 
            }
            0x1 => self.op_1nnn(),
            0x2 => self.op_2nnn(),
            0x3 => self.op_3xnn(),
            0x4 => self.op_4xnn(),
            0x5 => self.op_5xy0(),
            0x6 => self.op_6xnn(),
            0x7 => self.op_7xnn(),
            0x8 => match self.current_op.3 {
                0x0 => self.op_8xy0(),
                0x1 => self.op_8xy1(),
                0x2 => self.op_8xy2(),
                0x3 => self.op_8xy3(),
                0x4 => self.op_8xy4(),
                0x5 => self.op_8xy5(),
                0x6 => self.op_8xy6(),
                0x7 => self.op_8xy7(),
                0xE => self.op_8xye(),
                _ => self.unimplemented(),
            }
            0x9 => self.op_9xy0(),
            0xA => self.op_annn(),
            0xB => self.op_bnnn(),
            0xC => self.op_cxnn(),
            0xD => self.op_dxyn(),
            0xE => match self.current_op.2 {
                0x9 => self.op_ex9e(),
                0xA => self.op_exa1(),
                _ => self.unimplemented(),
            }
            0xF => match u16::from(self.current_op.2) << 4 | u16::from(self.current_op.3) {
                0x07 => self.op_fx07(),
                0x0A => self.op_fx0a(),
                0x15 => self.op_fx15(),
                0x18 => self.op_fx18(),
                0x1e => self.op_fx1e(),
                0x29 => self.op_fx29(),
                0x33 => self.op_fx33(),
                0x55 => self.op_fx55(),
                0x65 => self.op_fx65(),
                _ => self.unimplemented(),
            }
            _ => self.unimplemented(),
        }
    }

    fn unimplemented(&self) {
        let current_op: u16 = self.current_op.0 as u16 >> 12 |
                              self.current_op.1 as u16 >>  8 |
                              self.current_op.2 as u16 >>  4 |
                              self.current_op.3 as u16;
        panic!("Unimplemented Opcode: {:#06x}", current_op);
    }

    fn construct_address_from_op(&self) -> u16 {
        let addr = u16::from(self.current_op.1) << 8 |
                   u16::from(self.current_op.2) << 4 |
                   u16::from(self.current_op.3);
        addr
    }

    fn op_00ee(&mut self) { // return from subroutine
        match self.stack.pop() {
            Some(return_val) => self.pc = return_val,
            None => println!("Error executing 0x00ee: nothing on call stack!"),
        };
    }

    fn op_1nnn(&mut self) {
        let addr = self.construct_address_from_op();
        self.pc = addr;
    }

    fn op_2nnn(&mut self) {

    }

    fn op_3xnn(&mut self) {

    }

    fn op_4xnn(&mut self) {
    }

    fn op_5xy0(&mut self) {

    }

    fn op_6xnn(&mut self) {
    }

    fn op_7xnn(&mut self) {
    }

    fn op_8xy0(&mut self) {
    }

    fn op_8xy1(&mut self) {
    }

    fn op_8xy2(&mut self) {
    }

    fn op_8xy3(&mut self) {
    }

    fn op_8xy4(&mut self) {
    }

    fn op_8xy5(&mut self) {
    }

    fn op_8xy6(&mut self) {
    }

    fn op_8xy7(&mut self) {
    }

    fn op_8xye(&mut self) {
    }

    fn op_9xy0(&mut self) {
    }

    fn op_annn(&mut self) {
    }

    fn op_bnnn(&mut self) {
    }

    fn op_cxnn(&mut self) {
    }

    fn op_dxyn(&mut self) {
    }

    fn op_ex9e(&mut self) {
    }

    fn op_exa1(&mut self) {
    }

    fn op_fx07(&mut self) {
    }

    fn op_fx0a(&mut self) {
    }

    fn op_fx15(&mut self) {
    }

    fn op_fx18(&mut self) {
    }

    fn op_fx1e(&mut self) {
    }

    fn op_fx29(&mut self) {
    }

    fn op_fx33(&mut self) {
    }

    fn op_fx55(&mut self) {
    }

    fn op_fx65(&mut self) {
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
        assert_eq!((0,0,0,0), cpu.current_op);
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
        assert_eq!((0,0,0,0), cpu.current_op);
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
        assert_eq!(next_op, (0x1,0x2,0x4,0xe) );
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

    #[test]
    fn test_op_00ee() {
        let mut cpu = Cpu::new();
        cpu.stack.push(0x206);
        cpu.op_00ee();
        assert!(cpu.stack.is_empty());
        assert_eq!(cpu.pc, 0x206);
    }

    #[test]
    fn test_op_1nnn() {
        let mut cpu = Cpu::new();
        cpu.current_op = (0x1,0xA,0xB,0xC);
        cpu.op_1nnn();
        assert_eq!(cpu.pc, 0x0ABC);
    }
    
    #[test]
    fn test_construct_address_from_op() {
        let mut cpu = Cpu::new();
        cpu.current_op = (0x0, 0x1, 0x2, 0x3);
        let addr = cpu.construct_address_from_op();
        assert_eq!(addr, 0x0123);
    }
    
    #[test]
    fn test_op_2nnn_3xnn_4xnn_5xy0() {
        let mut cpu = Cpu::new();
        cpu.current_op = (0x2,0x2,0x2,0x2);
        cpu.op_2nnn();
        assert_eq!(cpu.stack[0], 0x0222);
        assert!( cpu.stack.len() == 1 );

        cpu.reset();
        let op1 = (0x3,0x1,0x2,0x2);
        let op2 = (0x3,0x1,0x4,0x4);

        cpu.v[0x1] = 0x44;
        cpu.v[0xE] = 0x43;
        cpu.current_op = op1;
        cpu.op_3xnn(); // since v[1] == 0x44, shouldn't skip.
        assert_eq!(cpu.pc, 0x200);
        cpu.op_4xnn(); // since v[1] != 0x22, should skip.
        assert_eq!(cpu.pc, 0x202);

        cpu.current_op = op2;
        cpu.op_3xnn(); // v1 == NN, skip
        assert_eq!(cpu.pc, 0x204);
        cpu.op_4xnn(); // v1 == NN, don't skip
        assert_eq!(cpu.pc, 0x204);

        cpu.current_op = (0x5,0x1,0xE,0x0);
        cpu.op_5xy0();
        assert_eq!(cpu.pc, 0x204);
        cpu.v[0xE] = 0x44;
        cpu.op_5xy0();
        assert_eq!(cpu.pc, 0x206);
    }

    #[test]
    fn test_op_6xnn() {
        let mut cpu = Cpu::new();
        cpu.current_op = (0x6, 0xD, 0xA, 0xD);
        cpu.op_6xnn();
        assert_eq!(cpu.v[0xD], 0xAD);
    }

    #[test]
    fn test_op_7xnn() {
        let mut cpu = Cpu::new();
        cpu.current_op = (0x7,0x2,0x0,0x3);
        cpu.v[2] = 3;
        let vf_state = cpu.v[0xF];
        cpu.op_7xnn();
        assert_eq!(cpu.v[2],6);
        cpu.v[2] = 253;
        cpu.op_7xnn();
        assert_eq!(cpu.v[2],0); // we should have overflowed here
        assert_eq!(cpu.v[0xF],vf_state); //...but carry shouldn't be affected
    }

    #[test]
    fn test_op_8xy0() {
        let mut cpu = Cpu::new();
        cpu.current_op = (8,0,1,0);
        cpu.v[0] = 40;
        cpu.v[1] = 0;
        cpu.op_8xy0();
        assert_eq!(cpu.v[0], cpu.v[1]);
    }

    #[test]
    fn test_op_8xy1() {
        let mut cpu = Cpu::new();
        cpu.current_op = (8,0,1,1);
        cpu.v[0] = 0x25;
        cpu.v[1] = 0x30;
        cpu.op_8xy1();
        assert_eq!(cpu.v[0], 0x35);
    }

    #[test]
    fn test_op_8xy2() {
        let mut cpu = Cpu::new();
        cpu.current_op = (8,0,1,2);
        cpu.v[0] = 0x25;
        cpu.v[1] = 0x30;
        cpu.op_8xy2();
        assert_eq!(cpu.v[0], 0x20);
    }

    #[test]
    fn test_op_8xy3() {
        let mut cpu = Cpu::new();
        cpu.current_op = (8,0,1,3);
        cpu.v[0] = 0x25;
        cpu.v[1] = 0x30;
        cpu.op_8xy3();
        assert_eq!(cpu.v[0], 0x15);
    }

    #[test]
    fn test_op_8xy4() {
        let mut cpu = Cpu::new();
        cpu.current_op = (8,0,1,4);
        cpu.v[0] = 250;
        cpu.v[1] = 3;
        cpu.op_8xy4();
        assert_eq!(cpu.v[0], 253);
        assert_eq!(cpu.v[0xF], 0);
        cpu.op_8xy4();
        assert_eq!(cpu.v[0], 0); //should have carried here
        assert_eq!(cpu.v[0], 1); //...and set the carry flag
    }

    #[test]
    fn test_op_8xy5() {
        let mut cpu = Cpu::new();
        cpu.current_op = (8,0,1,5);
        cpu.v[0] = 3;
        cpu.v[1] = 3;
        cpu.op_8xy5();
        assert_eq!(cpu.v[0], 0);
        assert_eq!(cpu.v[0xF], 0); //no borrow here
        cpu.op_8xy5();
        assert_eq!(cpu.v[0], 253);
        assert_eq!(cpu.v[0xF], 1); //we borrowed, carry flag should be set
    }

}
