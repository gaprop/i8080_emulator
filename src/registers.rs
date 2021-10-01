use std::ops::BitOr;

pub struct Registers {
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub f: u8, // Conditonal code or flag register
    pub a: u8,
}

pub enum Flag {
    S = 7, // Sign flag
    Z = 6, // Zero flag
    A = 4, // Auxillary carry flag
    P = 2, // Parity flag
    C = 0, // Carry flag
}

impl Registers {
    pub fn get_af(&self) -> u16 {
        (u16::from(self.a) << 8) | u16::from(self.f)
    }

    pub fn get_bc(&self) -> u16 {
        (u16::from(self.b) << 8) | u16::from(self.c)
    }

    pub fn get_de(&self) -> u16 {
        (u16::from(self.d) << 8) | u16::from(self.e)
    }

    pub fn get_hl(&self) -> u16 {
        (u16::from(self.h) << 8) | u16::from(self.l) 
    }

    pub fn set_af(&mut self, data: u16) {
        self.a = (data >> 8) as u8;
        self.f = (data & 0x00ff) as u8;
    }

    pub fn set_bc(&mut self, data: u16) {
        self.b = (data >> 8) as u8;
        self.c = (data & 0x00ff) as u8;
    }

    pub fn set_de(&mut self, data: u16) {
        self.d = (data >> 8) as u8;
        self.e = (data & 0x00ff) as u8;
    }

    pub fn set_hl(&mut self, data: u16) {
        self.h = (data >> 8) as u8; 
        self.l = (data & 0x00ff) as u8;
    }
}

impl Flag {
    pub fn is_flag(x: u8, f: Self) -> bool {
        let f = f as u8;
        x & (1 << f) == (1 << f)
    }
}

impl BitOr for Flag {
    type Output = u8;

    fn bitor(self, rhs: Self) -> u8 {
        let lhs = 1 << (self as u8);
        let rhs = 1 << (rhs as u8);
        lhs | rhs
    }
}

impl BitOr<Flag> for u8 {
    type Output = u8;

    fn bitor(self, rhs: Flag) -> u8 {
        let lhs = self;
        let rhs = 1 << (rhs as u8);
        lhs | rhs
    }
}

impl Registers {
    pub fn get_flag(&self, f: Flag) -> bool {
        (self.f >> (f as u8)) & 1 == 1
    }

    pub fn set_flag(&mut self, f: Flag, c: bool) {
        let f = f as u8;
        if c {
            self.f |= 1 << f
        } else {
            self.f &= (1 << f) ^ 0xff
        }
    }

    pub fn update_flags_from(&mut self, v: usize, f: u8) {
        if Flag::is_flag(f, Flag::S) {
            self.set_flag(Flag::S, v & 0xf0 == 0xf0);
        }
        if Flag::is_flag(f, Flag::Z) {
            self.set_flag(Flag::Z, v == 0);
        } 
        if Flag::is_flag(f, Flag::A) {
            self.set_flag(Flag::A, v > 0x0f);
        }
        if Flag::is_flag(f, Flag::P) {
            self.set_flag(Flag::P, v.count_ones() & 1 == 0);
        }
        if Flag::is_flag(f, Flag::C) {
            self.set_flag(Flag::C, v > 0xf0);
        }
    }
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            a: 0,
            f: 0x02,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::registers::{Registers, Flag};

    #[test]
    fn is_flag() {
        let x = 0x11;
        assert_eq!(Flag::is_flag(x, Flag::C), true);
    }

    #[test]
    fn flag_or() {
        assert_eq!(Flag::C | Flag::A, 0x11);
    }

    #[test]
    fn get_register_pair() {
        let mut regs = Registers::new();
        regs.a = 0xff;
        assert_eq!(regs.get_af(), 0xff02);
    }

    #[test]
    fn set_register_pair() {
        let mut regs = Registers::new();
        regs.set_hl(0xff02);
        assert_eq!(regs.h, 0xff);
        assert_eq!(regs.l, 0x02);
    }

    #[test]
    fn get_flag() {
        let mut regs = Registers::new();
        regs.f |= 0x80;
        assert_eq!(regs.get_flag(Flag::S), true);
    }

    #[test]
    fn set_flag() {
        let mut regs = Registers::new();
        regs.set_flag(Flag::S, true);
        regs.set_flag(Flag::P, false);
        assert_eq!(regs.f, 0b1000_0010);
    }

    #[test]
    fn update_flags_from() {
        let mut regs = Registers::new();
        let v: usize = 0x122;
        regs.update_flags_from(v, Flag::C | Flag::A);
        assert_eq!(regs.f, 0b0001_0011);
    }

    #[test]
    fn bitor_flags() {
        assert_eq!(Flag::S | Flag::Z | Flag::A, 208);
    }
}
