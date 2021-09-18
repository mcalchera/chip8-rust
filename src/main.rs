mod cpu;
use crate::cpu::Cpu;

#[cfg(not(tarpaulin_include))]
fn main() {
    println!("Hello, world!");
    let mut cpu = Cpu::new();
    assert!(cpu.load_rom("/home/mcalchera/development/rust/chip8-rust/test_opcode.ch8").is_ok());
    println!("{:?}",cpu);
}
