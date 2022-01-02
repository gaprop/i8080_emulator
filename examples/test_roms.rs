use i8080_emulator::cpu::CPU;
use i8080_emulator::memory::{Memory, Memory8080};
use i8080_emulator::device::Device;
use i8080_emulator::{Machine};

use std::cell::RefCell;
use std::rc::Rc;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::env;

struct Machine8080Test {
    cpu: CPU,
    memory: Rc<RefCell<Memory8080>>,
    test_finished: bool,
}

impl Machine8080Test {
    pub fn new(memory: [u8; 65536]) -> Self {
        let memory = Rc::new(RefCell::new(Memory8080::new(memory)));
        let mut cpu = CPU::new(Rc::clone(&memory));
        cpu.pc = 0x100;
        Machine8080Test {
            cpu,
            memory,
            test_finished: false,
        }
    }

}

impl Machine for Machine8080Test {
    fn next(&mut self) {
        let op = self.cpu.fetch();
        self.cpu.exec(op);

        if self.cpu.pc == 0x05 {
            let operation = self.cpu.regs.c;

            if operation == 2 {
                print!("{}", (self.cpu.regs.e) as char);
            } else if operation == 9 {
                let mut addr = self.cpu.regs.get_de();
                while (self.memory.borrow().read(addr.into()) as char) != '$' {
                    print!("{}", self.memory.borrow().read(addr.into()) as char);
                    addr += 1;
                }
            }
        }
        if self.cpu.pc == 0x00 {
            self.test_finished = true;
        }
    }

    fn run(&mut self) {
        while !self.test_finished {
            self.next();
        }
    }
}

pub fn read_file_into_buffer(path: impl AsRef<Path>, memory: &mut [u8; 0x10000], offset: usize) {
    let mut f = File::open(path).unwrap(); // I can not be bothered to actually handle this
    f.read_exact(&mut memory[offset..]);
}

fn main() {
    let filename = env::args().nth(1).expect("Needs a file");
    let mut memory = [0; 0x10000];
    read_file_into_buffer(filename, &mut memory, 0x100);

    memory[0x0005] = 0xc9;

    let mut machine = Machine8080Test::new(memory);
    println!("*********************");
    machine.run();
}
