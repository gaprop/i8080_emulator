pub mod cpu;
pub mod memory;
pub mod registers;
pub mod device;
pub mod disassembler;

use crate::device::Device;

pub trait Machine<T> {
     fn next(&mut self);
     fn run(&mut self);

     fn add_device(&mut self, port: usize, device: Box<dyn Device<T>>);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
