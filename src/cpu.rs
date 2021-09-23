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
// TODO: Remove this directive after writing main fn!
#[allow(dead_code)]
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
                    0xE0 => self.clear_screen(), // 0x00E0: clear screen
                    0xEE => self.op_00ee(),  // 0x00EE: return from subroutine
                    _ => self.unimplemented(),
                }
                _ => self.unimplemented(), // We don't support 0x0NNN instructions 
            }
            0x1 => self.op_1nnn(), // 1NNN: Jump to NNN
            0x2 => self.op_2nnn(), // 2NNN: call subroutine at NNN
            0x3 => self.op_3xnn(), // 3XNN: skip next instr if V[X] == NN
            0x4 => self.op_4xnn(), // 4XNN: skip next instr if V[X] != NN
            0x5 => self.op_5xy0(), // 5XY0: skip next instr if V[X] == V[Y]
            0x6 => self.op_6xnn(), // 6XNN: set V[X] == NN
            0x7 => self.op_7xnn(), // 7XNN: set V[X] += NN (carry flag not changed)
            0x8 => match self.current_op.3 {
                0x0 => self.op_8xy0(), // 8XY0: V[X] = V[Y]
                0x1 => self.op_8xy1(), // 8XY1: V[X] = V[X] OR V[Y]
                0x2 => self.op_8xy2(), // 8XY2: V[X] = V[X] AND V[Y]
                0x3 => self.op_8xy3(), // 8XY3: V[X] = V[X] XOR V[Y]
                0x4 => self.op_8xy4(), // 8XY4: V[X] = V[X] + V[Y] (carry flag set to 1 if carry, 0 if not)
                0x5 => self.op_8xy5(), // 8XY5: V[X] = V[X] - V[Y] (carry flag set to 0 if borrow, 1 if not)
                0x6 => self.op_8xy6(), // 8XY6: V[F] gets least sig bit of V[X], then VX >>= 1 (right shift)
                0x7 => self.op_8xy7(), // 8XY7: V[X] = V[Y] - V[X] (carry flag set to 0 if borrow, 1 if not)
                0xE => self.op_8xye(), // 8XYE: V[F] gets most sig bit of V[X], V[X] <<= 1 (left shift)
                _ => self.unimplemented(),
            }
            0x9 => self.op_9xy0(), // 9XY0: Skip next instr if V[X] != V[Y]
            0xA => self.op_annn(), // ANNN: Set index to NNN
            0xB => self.op_bnnn(), // BNNN: PC = V[0] + NNN
            0xC => self.op_cxnn(), // CXNN: V[X] = rand() AND NN 
            0xD => self.op_dxyn(), // DXYN: Draw sprite at (V[X],V[Y]), 8px wide x N high
                                   //       V[F] set to 1 if any screen pixels flipped from set to
                                   //       unset, 0 if not
            0xE => match self.current_op.2 {
                0x9 => self.op_ex9e(), // EX9E: skip next instr if key V[X] is pressed
                0xA => self.op_exa1(), // EXA1: skip next instr if key V[X] is NOT pressed
                _ => self.unimplemented(),
            }
            0xF => match u16::from(self.current_op.2) << 4 | u16::from(self.current_op.3) {
                0x07 => self.op_fx07(), // FX07: V[X] = value of delay timer
                0x0A => self.op_fx0a(), // FX0A: V[X] = key press (blocking wait for key press)
                0x15 => self.op_fx15(), // FX15: Set delay timer to V[X]
                0x18 => self.op_fx18(), // FX18: Set sound timer to V[X]
                0x1e => self.op_fx1e(), // FX1E: Index += V[X]. VF set to 1 if I + V[X} > 0xFFF, 0 if not
                0x29 => self.op_fx29(), // FX29: Index set to location of hex sprite corresponding to val of V[X]
                0x33 => self.op_fx33(), // FX33: Store binary-coded decimal representation of V[X] into:
                                        //   Index[0]: hundreds digit
                                        //   Index[1]: tens digit
                                        //   Index[2]: ones digit
                0x55 => self.op_fx55(), // FX55: store V[0] thru V[X] inclusive in memory starting at Index
                0x65 => self.op_fx65(), // FX65: load V[0] through V[X] inclusive from memory starting at Index
                _ => self.unimplemented(),
            }
            _ => self.unimplemented(),
        }
    }

    fn unimplemented(&self) {
        let current_op: u16 = (self.current_op.0 as u16) << 12 |
                              (self.current_op.1 as u16) <<  8 |
                              (self.current_op.2 as u16) <<  4 |
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
        let addr = self.construct_address_from_op();
        self.stack.push(self.pc);
        self.pc = addr;
    }

    fn op_3xnn(&mut self) {
        let x = self.current_op.1 as usize;
        let nn = self.current_op.2 << 4 | self.current_op.3;
        if self.v[x] == nn {
            self.pc += 2;
        }
    }

    fn op_4xnn(&mut self) {
        let x = self.current_op.1 as usize;
        let nn = self.current_op.2 << 4 | self.current_op.3;
        if self.v[x] != nn {
            self.pc += 2;
        }
    }

    fn op_5xy0(&mut self) {
        let x = self.current_op.1 as usize;
        let y = self.current_op.2 as usize;
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    fn op_6xnn(&mut self) {
        let x = self.current_op.1 as usize;
        let nn = self.current_op.2 << 4 | self.current_op.3;

        self.v[x] = nn;
    }

    fn op_7xnn(&mut self) {
        let x = self.current_op.1 as usize;
        let nn = self.current_op.2 << 4 | self.current_op.3;
        
        let result: u16 = self.v[x] as u16 + nn as u16;
        self.v[x] = result as u8;
    }

    fn op_8xy0(&mut self) {
        let x = self.current_op.1 as usize;
        let y = self.current_op.2 as usize;

        self.v[x] = self.v[y];
    }

    fn op_8xy1(&mut self) {
        let x = self.current_op.1 as usize;
        let y = self.current_op.2 as usize;

        self.v[x] = self.v[x] | self.v[y];
    }

    fn op_8xy2(&mut self) {
        let x = self.current_op.1 as usize;
        let y = self.current_op.2 as usize;
        
        self.v[x] = self.v[x] & self.v[y];
    }

    fn op_8xy3(&mut self) {
        let x = self.current_op.1 as usize;
        let y = self.current_op.2 as usize;

        self.v[x] = self.v[x] ^ self.v[y];
    }


    fn op_8xy4(&mut self) {
        let x = self.current_op.1 as usize;
        let y = self.current_op.2 as usize;

        let result: u16 = self.v[x] as u16 + self.v[y] as u16;
        if result > 0xFF {
            self.v[0xF] = 1;
        }
        else {
            self.v[0xF] = 0;
        }

        self.v[x] = result as u8;
    }

    fn op_8xy5(&mut self) {
        let x = self.current_op.1 as usize;
        let y = self.current_op.2 as usize;

        if self.v[x] < self.v[y] {
            self.v[0xF] = 0;
        } else {
            self.v[0xF] = 1;
        }

        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
    }

    fn op_8xy6(&mut self) {
        let x = self.current_op.1 as usize;
        self.v[0xF] = self.v[x] & 0x01;
        self.v[x] = self.v[x] >> 1;
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
        assert_eq!(cpu.stack[0], 0x200);
        assert_eq!(cpu.pc, 0x222);
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
        assert_eq!(cpu.v[0xF], 1); //...and set the carry flag
    }

    #[test]
    fn test_op_8xy5() {
        let mut cpu = Cpu::new();
        cpu.current_op = (8,0,1,5);
        cpu.v[0] = 3;
        cpu.v[1] = 3;
        cpu.op_8xy5();
        assert_eq!(cpu.v[0], 0);
        assert_eq!(cpu.v[0xF], 1); //no borrow here
        cpu.op_8xy5();
        assert_eq!(cpu.v[0], 253);
        assert_eq!(cpu.v[0xF], 0); //we borrowed, carry flag should not be set
    }

    #[test]
    fn test_op_8xy6() {
        let mut cpu = Cpu::new();
        let op1 = (8,0,2,6);
        let op2 = (8,1,2,6);
        cpu.v[0] = 0x80; // right shifting this will not set V[F]
        cpu.v[1] = 0x01; // right shifting this should set V[F]
        cpu.current_op = op1;
        cpu.op_8xy6();
        assert_eq!(cpu.v[0], 0x40);
        assert_eq!(cpu.v[0xF], 0);
        cpu.current_op = op2;
        cpu.op_8xy6();
        assert_eq!(cpu.v[1], 0x0);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn test_op_8xy7() {
        // This test is the same as 8xy5, just subtracting y from x instead
        // As such, i am using the same test as 8xy5 with the registers reversed
        let mut cpu = Cpu::new();
        cpu.current_op = (8,1,0,7);
        cpu.v[0] = 3;
        cpu.v[1] = 3;
        cpu.op_8xy7();
        assert_eq!(cpu.v[1], 0);
        assert_eq!(cpu.v[0xF], 1); //no borrow here
        cpu.op_8xy7();
        assert_eq!(cpu.v[1], 253);
        assert_eq!(cpu.v[0xF], 0); //we borrowed, carry flag should not be set
    }
    
    #[test]
    fn test_op_8xye() {
        // ditto for this test.  Very similar to 8xy6 but with left shifts
        let mut cpu = Cpu::new();
        let op1 = (8,0,2,0xE);
        let op2 = (8,1,2,0xE);
        cpu.v[0] = 0x80; // 0x80 << 1 = 0x00, V[F] = 1
        cpu.v[1] = 0x01; // 0x01 << 1 = 0x02, V[F] = 0
        cpu.current_op = op1;
        cpu.op_8xye();
        assert_eq!(cpu.v[0], 0x00);
        assert_eq!(cpu.v[0xF], 1);
        cpu.current_op = op2;
        cpu.op_8xye();
        assert_eq!(cpu.v[1], 0x02);
        assert_eq!(cpu.v[0xF], 0);
    }

    #[test]
    fn test_op_9xy0() {
        let mut cpu = Cpu::new();
        let op1 = (9,0,0xE,0x0);
        let op2 = (9,1,0xE,0x0);
        cpu.v[0] = 0x55;
        cpu.v[0xE] = 0x55;
        cpu.pc = 0x400; // changing program counter from default
        
        cpu.current_op = op1;
        cpu.op_9xy0();
        assert_eq!(cpu.pc, 0x400);
        cpu.current_op = op2;
        cpu.v[0] = 0x54;
        cpu.op_9xy0();
        assert_eq!(cpu.pc, 0x402);
    }

    #[test]
    fn test_op_annn() {
        let mut cpu = Cpu::new();
        cpu.current_op = (0xA,4,5,6);
        cpu.op_annn();
        assert_eq!(cpu.index, 0x456);
    }

    #[test]
    fn test_op_bnnn() {
        let mut cpu = Cpu::new();
        cpu.current_op = (0xB,3,0,0);
        cpu.v[0] = 5;
        cpu.op_bnnn();
        assert_eq!(cpu.pc, 0x305);
    }

    #[test]
    fn test_op_cxnn() {
        let mut cpu = Cpu::new();
        cpu.current_op = (0xC,1,2,4);
        cpu.v[1] = 0x20;
        cpu.op_cxnn();
        assert!(cpu.v[1] <= 0x20); // kinda hard to test random results...
    }

    #[test]
    fn test_op_dxyn() {
        let mut cpu = Cpu::new();
        cpu.index = 0x300;
        let i = cpu.index as usize;
        cpu.memory[i]   = 0b1111_1111;
        cpu.memory[i+1] = 0b0000_0000;
        cpu.graphics[0][0] = 0;
        cpu.graphics[0][1] = 1; // will be flipped
        cpu.graphics[1][0] = 0; 
        cpu.graphics[1][1] = 1; // will be flipped
        
        cpu.current_op = (0xD,0,0,2); 
        assert_eq!(cpu.graphics[0][0], 1);
        assert_eq!(cpu.graphics[0][1], 0);
        assert_eq!(cpu.graphics[1][0], 0);
        assert_eq!(cpu.graphics[1][1], 1);
        assert_eq!(cpu.v[0xF], 1);
    }

    #[test]
    fn test_op_ex9e() {
        let mut cpu = Cpu::new();
        cpu.current_op = (0xE,0,0x9,0xE);
        cpu.v[0] = 5;
        cpu.key_pressed[5] = 1;
        cpu.op_ex9e();
        assert_eq!(cpu.pc, 0x202);
        cpu.key_pressed[5] = 0;
        cpu.op_ex9e();
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_op_exa1() {
        let mut cpu = Cpu::new();
        cpu.current_op = (0xE,0,0xA,0x1);
        cpu.v[0] = 5;
        cpu.key_pressed[5] = 1;
        cpu.op_ex9e();
        assert_eq!(cpu.pc, 0x200);
        cpu.key_pressed[5] = 0;
        cpu.op_ex9e();
        assert_eq!(cpu.pc, 0x202);
    }

    #[test]
    fn test_op_fx07() {
        let mut cpu = Cpu::new();
        cpu.current_op = (0xF,0,0,7);
        cpu.v[0] = 0x2;
        cpu.delay_timer = 0xEF;
        cpu.op_fx07();
        assert_eq!(cpu.v[0], 0xEF);
    }
}
