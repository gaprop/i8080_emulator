pub trait Memory {
     fn read(&self, i: usize) -> u8;
     fn write(&mut self, i: usize, data: u8);

     fn read16(&self, i: usize) -> u16;
     fn write16(&mut self, i: usize, data: u16);
}

pub struct Memory8080 {
    memory: [u8; 65536]
}

impl Memory for Memory8080 {
    fn read(&self, i: usize) -> u8 {
        self.memory[i]
    }

    fn write(&mut self, i: usize, data: u8) {
        self.memory[i] = data;
    }

    fn read16(&self, i: usize) -> u16 {
        let hi = self.read(i + 1);
        let lo = self.read(i);

        (u16::from(hi) << 8) | u16::from(lo)
    }

    fn write16(&mut self, i: usize, data: u16) {
        let hi = ((data & 0xff00) >> 8) as u8;
        let lo = (data & 0xff) as u8;

        self.write(i + 1, hi);
        self.write(i, lo);
    }
}

impl Memory8080 {
    pub fn new_empty() -> Self {
        Memory8080 {
            memory: [0; 65536]
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::{Memory8080, Memory};

    #[test]
    fn read() {
        let mut memory = Memory8080::new_empty();
        memory.memory[0] = 0xff;
        memory.memory[1] = 0x02;
        assert_eq!(memory.read(0), 0xff);
    }

    #[test]
    fn write() {
        let mut memory = Memory8080::new_empty();
        memory.write(4, 0xff);
        assert_eq!(memory.memory[4], 0xff);
    }

    #[test]
    fn read16() {
        let mut memory = Memory8080::new_empty();
        memory.memory[0] = 0xff;
        memory.memory[1] = 0x02;
        assert_eq!(memory.read16(0), 0xff02);
    }

    #[test]
    fn write16() {
        let mut memory = Memory8080::new_empty();
        memory.write16(4, 0xff02);
        assert_eq!(memory.memory[4], 0xff);
        assert_eq!(memory.memory[5], 0x02);
    }
}
