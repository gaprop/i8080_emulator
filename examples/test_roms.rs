use i8080_emulator::cpu::{Event, CPU};
use i8080_emulator::device::Device;
use i8080_emulator::{Machine};

struct Machine8080 {
    cpu: CPU,
    devices: Vec<Box<dyn Device<Event>>>,
}

impl Machine8080 {
    pub fn new() -> Self {
        Machine8080 {
            cpu: CPU::new_empty(),
            devices: vec![],
        }
    }
}

impl Machine<Event> for Machine8080 {
    fn add_device(&mut self, port: usize, device: Box<dyn Device<Event>>) {
        self.devices.insert(port, device);
    }

    fn next(&self) {
        println!("...");
    }

    fn run(&self) {
        println!("...");
    }
}

fn main() {
    let cpu = CPU::new_empty();
    let cpu2 = CPU::new_empty();
    let mut mac = Machine8080 { cpu, devices: vec![], };
    mac.add_device(0, Box::new(cpu2));
}
