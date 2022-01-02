pub mod cpu;
pub mod memory;
pub mod registers;
pub mod device;
pub mod disassembler;

pub trait Machine {
     fn next(&mut self);
     fn run(&mut self);
}
