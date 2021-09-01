use i8080_emulator::cpu::{Event, CPU};
use i8080_emulator::memory::{Memory};
use i8080_emulator::device::Device;
use i8080_emulator::{Machine};

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::env;

struct Machine8080Test {
    cpu: CPU,
    devices: Vec<Box<dyn Device<Event>>>,
    test_finished: bool,
}

impl Machine8080Test {
    pub fn new(memory: [u8; 65536]) -> Self {
        let mut cpu = CPU::new(memory);
        cpu.pc = 0x100;
        Machine8080Test {
            cpu,
            devices: vec![],
            test_finished: false,
        }
    }

}

impl Machine<Event> for Machine8080Test {
    fn add_device(&mut self, port: usize, device: Box<dyn Device<Event>>) {
        self.devices.insert(port, device);
    }

    fn next(&mut self) {
        let op = self.cpu.fetch();
        let event = self.cpu.exec(op);

        match event {
            Event::Output(port, _, _) => {
                if port == 0 {
                    self.test_finished = true;
                }

                if port == 1 {
                    let operation = self.cpu.regs.c;

                    if operation == 2 {
                        print!("{}", (self.cpu.regs.e) as char);
                    } else if operation == 9 {
                        let mut addr = self.cpu.regs.get_de();
                        while (self.cpu.memory.read(addr.into()) as char) != '$' {
                            print!("{}", self.cpu.memory.read(addr.into()) as char);
                            addr += 1;
                        }
                    }
                }
            }
            _ => (),
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
    memory[0x0000] = 0xd3;
    memory[0x0001] = 0x00;

    memory[0x0005] = 0xd3;
    memory[0x0006] = 0x01;
    memory[0x0007] = 0xc9;

    let mut machine = Machine8080Test::new(memory);
    println!("*********************");
    machine.run();
}
