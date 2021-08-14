use crate::memory::{Memory, Memory8080};
use crate::registers::{Registers, Flag};
use crate::device::Device;

type ClockCycles = u8;
type Port = u8;

pub enum Event {
    Output(Port, u8, ClockCycles),
    Halt(ClockCycles),
    Normal(ClockCycles),
}

pub struct CPU {
    pub regs: Registers,
    pub memory: Memory8080,
    pub pc: u16,
    sp: u16,
    inte: bool,
}

impl CPU {
    pub fn new_empty() -> Self {
        CPU {
            regs: Registers::new(),
            memory: Memory8080::new_empty(),
            pc: 0,
            sp: 0xf000,
            inte: false,
        }
    }

    fn get_m(&self) -> u8 {
        self.memory.read(self.regs.get_hl().into())
    }

    fn set_m(&mut self, data: u8) {
        self.memory.write(self.regs.get_hl().into(), data);
    }

    //// Instruction functions

    // Store instructions
    fn stax(&mut self, addr: u16) {
        let addr = addr as usize;
        self.memory.write(addr, self.regs.a);
    }

    // Arithmetic instructions
    fn inr(&mut self, mut regm: u8) -> u8 {
        let mut n: usize = regm as usize;
        n = n.wrapping_add(1);
        regm = regm.wrapping_add(1);
        self.regs.update_flags_from(n, Flag::S | Flag::Z | Flag::A | Flag::P);
        regm
    }

    fn dcr(&mut self, mut regm: u8) -> u8 {
        let mut n: usize = regm as usize;
        n = n.wrapping_sub(1);
        regm = regm.wrapping_sub(1);
        self.regs.update_flags_from(n, Flag::S | Flag::Z | Flag::A | Flag::P);
        regm
    }

    fn add(&mut self, mut regm1: u8, regm2: u8) -> u8 {
        let mut n1: usize = regm1 as usize;
        let n2: usize = regm2 as usize;
        n1 = n1.wrapping_add(n2);
        regm1 = regm1.wrapping_add(regm2);
        self.regs.update_flags_from(n1, Flag::S | Flag::Z | Flag::A | Flag::P | Flag::C);
        regm1
    }

    fn adc(&mut self, mut regm1: u8, regm2: u8) -> u8 {
        let mut n1: usize = regm1 as usize;
        let n2: usize = regm2 as usize;
        let carry: u8 = if self.regs.get_flag(Flag::C) { 1 } else { 0 };

        n1 = n1.wrapping_add(n2).wrapping_add(carry as usize);
        regm1 = regm1.wrapping_add(regm2).wrapping_add(carry);
        self.regs.update_flags_from(n1, Flag::S | Flag::Z | Flag::A | Flag::P | Flag::C);
        regm1
    }

    fn sub(&mut self, mut regm1: u8, regm2: u8) -> u8 {
        let mut n1: usize = regm1 as usize;
        let n2: usize = regm2 as usize;
        n1 = n1.wrapping_sub(n2);
        regm1 = regm1.wrapping_sub(regm2);
        self.regs.update_flags_from(n1, Flag::S | Flag::Z | Flag::A | Flag::P | Flag::C);
        regm1
    }

    fn sbb(&mut self, mut regm1: u8, regm2: u8) -> u8 {
        let mut n1: usize = regm1 as usize;
        let n2: usize = regm2 as usize;
        let carry: u8 = if self.regs.get_flag(Flag::C) { 1 } else { 0 };

        n1 = n1.wrapping_sub(n2).wrapping_sub(carry as usize);
        regm1 = regm1.wrapping_sub(regm2).wrapping_sub(carry);
        self.regs.update_flags_from(n1, Flag::S | Flag::Z | Flag::A | Flag::P | Flag::C);
        regm1
    }

    // Bitwise instructions
    fn ana(&mut self, mut regm1: u8, regm2: u8) -> u8 {
        regm1 &= regm2;
        self.regs.update_flags_from(regm1 as usize, Flag::S | Flag::Z | Flag::A | Flag::P | Flag::C);
        regm1
    }

    fn xra(&mut self, mut regm1: u8, regm2: u8) -> u8 {
        regm1 ^= regm2;
        self.regs.update_flags_from(regm1 as usize, Flag::S | Flag::Z | Flag::A | Flag::P | Flag::C);
        regm1
    }

    fn ora(&mut self, mut regm1: u8, regm2: u8) -> u8 {
        regm1 |= regm2;
        self.regs.update_flags_from(regm1 as usize, Flag::S | Flag::Z | Flag::A | Flag::P | Flag::C);
        regm1
    }

    fn cmp(&mut self, regm1: u8, regm2: u8) {
        let regm1: usize = regm1 as usize;
        let regm2: usize = regm2 as usize;
        let n: usize = regm1 - regm2;
        self.regs.update_flags_from(n, Flag::S | Flag::Z | Flag::A | Flag::P | Flag::C);
    }
    // Jump instructions
    fn jmp(&mut self, cond: bool) {
        if cond {
            let addr = self.memory.read16(self.pc.into());
            self.pc = addr;
        } else {
            self.pc = self.pc.wrapping_add(2);
        }
    }

    fn call(&mut self, cond: bool) -> Event {
        if cond {
            self.memory.write16(self.sp.wrapping_sub(2).into(), 
                                self.pc.wrapping_add(2));
            self.sp = self.sp.wrapping_sub(2);

            let addr = self.memory.read16(self.pc.into());
            self.pc = addr;
            Event::Normal(17)
        } else {
            self.pc = self.pc.wrapping_add(2);
            Event::Normal(11)
        }
    }

    fn ret(&mut self, cond: bool) -> Event {
        if cond {
            let addr = self.memory.read16(self.sp.into());
            self.sp = self.sp.wrapping_add(2);
            self.pc = addr;

            Event::Normal(11)
        } else {
            Event::Normal(5)
        }
    }

    fn push(&mut self, data: u16) {
        self.sp = self.sp.wrapping_sub(2);
        self.memory.write16(data.into(), self.sp);
    }

    fn pop(&mut self) -> u16 {
        let data = self.memory.read16(self.sp.into());
        self.sp = self.sp.wrapping_add(2);
        data
    }

    fn rst(&mut self, addr: u16) {
        self.memory.write16(self.sp.wrapping_add(2).into(),
                            self.pc);
        self.pc = 0x0000 | addr;
    }
}

impl Device<Event> for CPU {
    fn fetch(&mut self) -> u8 {
        let op = self.memory.read(self.pc.into());
        self.pc += 1;
        op
    }

    fn exec(&mut self, op: u8) -> Event {
        match op {
            // NOP
            0x00 => Event::Normal(4),
            0x10 => Event::Normal(4),
            0x20 => Event::Normal(4),
            0x30 => Event::Normal(4),
            0x08 => Event::Normal(4),
            0x18 => Event::Normal(4),
            0x28 => Event::Normal(4),
            0x38 => Event::Normal(4),

            // LXI
            0x01 => { 
                let data = self.memory.read16(self.pc.into());
                self.pc += 2;
                self.regs.set_bc(data);
                Event::Normal(10)
            }
            0x11 => {
                let data = self.memory.read16(self.pc.into());
                self.pc += 2;
                self.regs.set_de(data);
                Event::Normal(10)
            }
            0x21 => {
                let data = self.memory.read16(self.pc.into());
                self.pc += 2;
                self.regs.set_hl(data);
                Event::Normal(10)
            }
            0x31 => {
                let data = self.memory.read16(self.pc.into());
                self.pc += 2;
                self.sp = data;
                Event::Normal(10)
            }

            // STAX
            0x02 => { self.stax(self.regs.get_bc()); Event::Normal(7) },
            0x12 => { self.stax(self.regs.get_de()); Event::Normal(7) },

            // INX
            0x03 => { self.regs.set_bc(self.regs.get_bc() + 1); Event::Normal(5) }
            0x13 => { self.regs.set_de(self.regs.get_de() + 1); Event::Normal(5) }
            0x23 => { self.regs.set_hl(self.regs.get_hl() + 1); Event::Normal(5) }
            0x33 => { self.sp += 1; Event::Normal(5) }

            // INR
            0x04 => { self.regs.b = self.inr(self.regs.b); Event::Normal(5) }
            0x0c => { self.regs.c = self.inr(self.regs.c); Event::Normal(5) }
            0x14 => { self.regs.d = self.inr(self.regs.d); Event::Normal(5) }
            0x1c => { self.regs.e = self.inr(self.regs.e); Event::Normal(5) }
            0x24 => { self.regs.h = self.inr(self.regs.h); Event::Normal(5) }
            0x2c => { self.regs.l = self.inr(self.regs.l); Event::Normal(5) }
            0x34 => { 
                let n = self.inr(self.get_m());
                self.set_m(n); 
                Event::Normal(10) 
            }
            0x3c => { self.regs.a = self.inr(self.regs.a); Event::Normal(5) }

            // DCR
            0x05 => { self.regs.b = self.dcr(self.regs.b); Event::Normal(5) }
            0x0d => { self.regs.c = self.dcr(self.regs.c); Event::Normal(5) }
            0x15 => { self.regs.d = self.dcr(self.regs.d); Event::Normal(5) }
            0x1d => { self.regs.e = self.dcr(self.regs.e); Event::Normal(5) }
            0x25 => { self.regs.h = self.dcr(self.regs.h); Event::Normal(5) }
            0x2d => { self.regs.l = self.dcr(self.regs.l); Event::Normal(5) }
            0x35 => { 
                let n = self.dcr(self.get_m());
                self.set_m(n); 
                Event::Normal(10) 
            }
            0x3d => { self.regs.a = self.dcr(self.regs.a); Event::Normal(5) }

            // DCX
            0x0b => { self.regs.set_bc(self.regs.get_bc() - 1); Event::Normal(5) }
            0x1b => { self.regs.set_de(self.regs.get_de() - 1); Event::Normal(5) }
            0x2b => { self.regs.set_hl(self.regs.get_hl() - 1); Event::Normal(5) }
            0x3b => { self.sp -= 1; Event::Normal(5) }

            // ADD
            0x80 => { self.regs.a = self.add(self.regs.a, self.regs.b); Event::Normal(4) }
            0x81 => { self.regs.a = self.add(self.regs.a, self.regs.c); Event::Normal(4) }
            0x82 => { self.regs.a = self.add(self.regs.a, self.regs.d); Event::Normal(4) }
            0x83 => { self.regs.a = self.add(self.regs.a, self.regs.e); Event::Normal(4) }
            0x84 => { self.regs.a = self.add(self.regs.a, self.regs.h); Event::Normal(4) }
            0x85 => { self.regs.a = self.add(self.regs.a, self.regs.l); Event::Normal(4) }
            0x86 => { self.regs.a = self.add(self.regs.a, self.get_m()); Event::Normal(7) }
            0x87 => { self.regs.a = self.add(self.regs.a, self.regs.a); Event::Normal(4) }

            // SUB
            0x90 => { self.regs.a = self.sub(self.regs.a, self.regs.b); Event::Normal(4) }
            0x91 => { self.regs.a = self.sub(self.regs.a, self.regs.c); Event::Normal(4) }
            0x92 => { self.regs.a = self.sub(self.regs.a, self.regs.d); Event::Normal(4) }
            0x93 => { self.regs.a = self.sub(self.regs.a, self.regs.e); Event::Normal(4) }
            0x94 => { self.regs.a = self.sub(self.regs.a, self.regs.h); Event::Normal(4) }
            0x95 => { self.regs.a = self.sub(self.regs.a, self.regs.l); Event::Normal(4) }
            0x96 => { self.regs.a = self.sub(self.regs.a, self.get_m()); Event::Normal(7) }
            0x97 => { self.regs.a = self.sub(self.regs.a, self.regs.a); Event::Normal(4) }

            // ADC
            0x88 => { self.regs.a = self.adc(self.regs.a, self.regs.b); Event::Normal(4) }
            0x89 => { self.regs.a = self.adc(self.regs.a, self.regs.c); Event::Normal(4) }
            0x8a => { self.regs.a = self.adc(self.regs.a, self.regs.d); Event::Normal(4) }
            0x8b => { self.regs.a = self.adc(self.regs.a, self.regs.e); Event::Normal(4) }
            0x8c => { self.regs.a = self.adc(self.regs.a, self.regs.h); Event::Normal(4) }
            0x8d => { self.regs.a = self.adc(self.regs.a, self.regs.l); Event::Normal(4) }
            0x8e => { self.regs.a = self.adc(self.regs.a, self.get_m()); Event::Normal(7) }
            0x8f => { self.regs.a = self.adc(self.regs.a, self.regs.a); Event::Normal(4) }

            // SBB
            0x98 => { self.regs.a = self.sbb(self.regs.a, self.regs.b); Event::Normal(4) }
            0x99 => { self.regs.a = self.sbb(self.regs.a, self.regs.c); Event::Normal(4) }
            0x9a => { self.regs.a = self.sbb(self.regs.a, self.regs.d); Event::Normal(4) }
            0x9b => { self.regs.a = self.sbb(self.regs.a, self.regs.e); Event::Normal(4) }
            0x9c => { self.regs.a = self.sbb(self.regs.a, self.regs.h); Event::Normal(4) }
            0x9d => { self.regs.a = self.sbb(self.regs.a, self.regs.l); Event::Normal(4) }
            0x9e => { self.regs.a = self.sbb(self.regs.a, self.get_m()); Event::Normal(7) }
            0x9f => { self.regs.a = self.sbb(self.regs.a, self.regs.a); Event::Normal(4) }

            // ANA
            0xa0 => { self.regs.a = self.ana(self.regs.a, self.regs.b); Event::Normal(4) }
            0xa1 => { self.regs.a = self.ana(self.regs.a, self.regs.c); Event::Normal(4) }
            0xa2 => { self.regs.a = self.ana(self.regs.a, self.regs.d); Event::Normal(4) }
            0xa3 => { self.regs.a = self.ana(self.regs.a, self.regs.e); Event::Normal(4) }
            0xa4 => { self.regs.a = self.ana(self.regs.a, self.regs.h); Event::Normal(4) }
            0xa5 => { self.regs.a = self.ana(self.regs.a, self.regs.l); Event::Normal(4) }
            0xa6 => { self.regs.a = self.ana(self.regs.a, self.get_m()); Event::Normal(7) }
            0xa7 => { self.regs.a = self.ana(self.regs.a, self.regs.a); Event::Normal(4) }

            // XRA
            0xa8 => { self.regs.a = self.xra(self.regs.a, self.regs.b); Event::Normal(4) }
            0xa9 => { self.regs.a = self.xra(self.regs.a, self.regs.c); Event::Normal(4) }
            0xaa => { self.regs.a = self.xra(self.regs.a, self.regs.d); Event::Normal(4) }
            0xab => { self.regs.a = self.xra(self.regs.a, self.regs.e); Event::Normal(4) }
            0xac => { self.regs.a = self.xra(self.regs.a, self.regs.h); Event::Normal(4) }
            0xad => { self.regs.a = self.xra(self.regs.a, self.regs.l); Event::Normal(4) }
            0xae => { self.regs.a = self.xra(self.regs.a, self.get_m()); Event::Normal(7) }
            0xaf => { self.regs.a = self.xra(self.regs.a, self.regs.a); Event::Normal(4) }

            // ORA
            0xb0 => { self.regs.a = self.ora(self.regs.a, self.regs.b); Event::Normal(4) }
            0xb1 => { self.regs.a = self.ora(self.regs.a, self.regs.c); Event::Normal(4) }
            0xb2 => { self.regs.a = self.ora(self.regs.a, self.regs.d); Event::Normal(4) }
            0xb3 => { self.regs.a = self.ora(self.regs.a, self.regs.e); Event::Normal(4) }
            0xb4 => { self.regs.a = self.ora(self.regs.a, self.regs.h); Event::Normal(4) }
            0xb5 => { self.regs.a = self.ora(self.regs.a, self.regs.l); Event::Normal(4) }
            0xb6 => { self.regs.a = self.ora(self.regs.a, self.get_m()); Event::Normal(7) }
            0xb7 => { self.regs.a = self.ora(self.regs.a, self.regs.a); Event::Normal(4) }

            // CMP
            0xb8 => { self.cmp(self.regs.a, self.regs.b); Event::Normal(4) }
            0xb9 => { self.cmp(self.regs.a, self.regs.c); Event::Normal(4) }
            0xba => { self.cmp(self.regs.a, self.regs.d); Event::Normal(4) }
            0xbb => { self.cmp(self.regs.a, self.regs.e); Event::Normal(4) }
            0xbc => { self.cmp(self.regs.a, self.regs.h); Event::Normal(4) }
            0xbd => { self.cmp(self.regs.a, self.regs.l); Event::Normal(4) }
            0xbe => { self.cmp(self.regs.a, self.get_m()); Event::Normal(7) }
            0xbf => { self.cmp(self.regs.a, self.regs.a); Event::Normal(4) }

            // ADI
            0xc6 => { 
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.add(self.regs.a, data);
                Event::Normal(7)
            }

            // ACI
            0xce => { 
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.adc(self.regs.a, data);
                Event::Normal(7)
            }

            // SUI
            0xd6 => { 
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.sub(self.regs.a, data);
                Event::Normal(7)
            }

            // SBI
            0xde => { 
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.sbb(self.regs.a, data);
                Event::Normal(7)
            }

            // ANI
            0xe6 => {
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.ana(self.regs.a, data);
                Event::Normal(7)
            }

            // XRI
            0xee => {
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.xra(self.regs.a, data);
                Event::Normal(7)
            }

            // ORI
            0xf6 => {
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.ora(self.regs.a, data);
                Event::Normal(7)
            }

            // CPI
            0xfe => {
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.cmp(self.regs.a, data);
                Event::Normal(7)
            }

            // RLC
            0x07 => {
                let mut n: usize = self.regs.a.into();
                n = n << 1;
                self.regs.a = self.regs.a << 1;
                self.regs.update_flags_from(n, Flag::C as u8);
                Event::Normal(4)
            }

            // RRC
            0x0f => {
                let lo = self.regs.a & 1;
                self.regs.a = self.regs.a >> 1;
                if lo == 1 {
                    self.regs.set_flag(Flag::C, true);
                }
                Event::Normal(4)
            }

            // RAL
            0x17 => {
                let mut n: usize = self.regs.a.into();
                let carry: u8 = if self.regs.get_flag(Flag::C) { 0x80 } else { 0 };
                self.regs.a = self.regs.a << 1;
                self.regs.a |= carry;
                n = n << 1;
                self.regs.update_flags_from(n, Flag::C as u8);
                Event::Normal(4)
            }

            // RAR
            0x1f => {
                let lo = self.regs.a & 1;
                let carry: u8 = if self.regs.get_flag(Flag::C) { 0x80 } else { 0 };
                self.regs.a = self.regs.a >> 1;
                self.regs.a |= carry;
                if lo == 1 {
                    self.regs.set_flag(Flag::C, true);
                }
                Event::Normal(4)
            }

            // CMA
            0x2f => {
                self.regs.a ^= self.regs.a;
                Event::Normal(4)
            }

            // CMC
            0x3f => {
                let carry = self.regs.get_flag(Flag::C);
                self.regs.set_flag(Flag::C, !carry);
                Event::Normal(4)
            }

            // DAA FIXME: propably has errors
            0x27 => {
                let lo: u8 = self.regs.a & 0x0f;
                if lo > 9 || self.regs.get_flag(Flag::A) {
                    self.regs.a = self.regs.a.wrapping_add(9);
                    self.regs.update_flags_from(self.regs.a as usize, Flag::A as u8);
                }

                let mut hi: usize = ((self.regs.a & 0xf0) >> 7).into();
                if hi > 9 || self.regs.get_flag(Flag::C) {
                    hi += 9;
                    hi = hi << 8;
                    self.regs.a = self.regs.a | (hi as u8);
                    self.regs.update_flags_from(hi, Flag::C as u8);
                }
                Event::Normal(4)
            }

            // STC
            0x37 => { self.regs.set_flag(Flag::C, true); Event::Normal(4) }

            // DAD
            0x09 => { 
                let n: usize = (self.regs.get_bc() + self.regs.get_hl()).into();
                self.regs.set_bc(n as u16); 
                self.regs.update_flags_from(n, Flag::C as u8);
                Event::Normal(10) 
            }
            0x19 => { 
                let n: usize = (self.regs.get_de() + self.regs.get_hl()).into();
                self.regs.set_de(n as u16); 
                self.regs.update_flags_from(n, Flag::C as u8);
                Event::Normal(10) 
            }
            0x29 => { 
                let n: usize = (self.regs.get_hl() + self.regs.get_hl()).into();
                self.regs.set_hl(n as u16); 
                self.regs.update_flags_from(n, Flag::C as u8);
                Event::Normal(10) 
            }
            0x39 => { 
                let n: usize = (self.sp as usize) + (self.regs.get_hl() as usize);
                self.sp = n as u16;
                self.regs.update_flags_from(n, Flag::C as u8);
                Event::Normal(10) 
            }

            // MOV B regm
            0x40 => { self.regs.b = self.regs.b; Event::Normal(5) }
            0x41 => { self.regs.b = self.regs.c; Event::Normal(5) }
            0x42 => { self.regs.b = self.regs.d; Event::Normal(5) }
            0x43 => { self.regs.b = self.regs.e; Event::Normal(5) }
            0x44 => { self.regs.b = self.regs.h; Event::Normal(5) }
            0x45 => { self.regs.b = self.regs.l; Event::Normal(5) }
            0x46 => { self.regs.b = self.get_m(); Event::Normal(7) }
            0x47 => { self.regs.b = self.regs.a; Event::Normal(5) }

            // MOV C regsm
            0x48 => { self.regs.c = self.regs.b; Event::Normal(5) }
            0x49 => { self.regs.c = self.regs.c; Event::Normal(5) }
            0x4a => { self.regs.c = self.regs.d; Event::Normal(5) }
            0x4b => { self.regs.c = self.regs.e; Event::Normal(5) }
            0x4c => { self.regs.c = self.regs.h; Event::Normal(5) }
            0x4d => { self.regs.c = self.regs.l; Event::Normal(5) }
            0x4e => { self.regs.c = self.get_m(); Event::Normal(7) }
            0x4f => { self.regs.c = self.regs.a; Event::Normal(5) }

            // MOV D regsm
            0x50 => { self.regs.d = self.regs.b; Event::Normal(5) }
            0x51 => { self.regs.d = self.regs.c; Event::Normal(5) }
            0x52 => { self.regs.d = self.regs.d; Event::Normal(5) }
            0x53 => { self.regs.d = self.regs.e; Event::Normal(5) }
            0x54 => { self.regs.d = self.regs.h; Event::Normal(5) }
            0x55 => { self.regs.d = self.regs.l; Event::Normal(5) }
            0x56 => { self.regs.d = self.get_m(); Event::Normal(7) }
            0x57 => { self.regs.d = self.regs.a; Event::Normal(5) }

            // MOV E regsm
            0x58 => { self.regs.e = self.regs.b; Event::Normal(5) }
            0x59 => { self.regs.e = self.regs.c; Event::Normal(5) }
            0x5a => { self.regs.e = self.regs.d; Event::Normal(5) }
            0x5b => { self.regs.e = self.regs.e; Event::Normal(5) }
            0x5c => { self.regs.e = self.regs.h; Event::Normal(5) }
            0x5d => { self.regs.e = self.regs.l; Event::Normal(5) }
            0x5e => { self.regs.e = self.get_m(); Event::Normal(7) }
            0x5f => { self.regs.e = self.regs.a; Event::Normal(5) }

            // MOV H regsm
            0x60 => { self.regs.h = self.regs.b; Event::Normal(5) }
            0x61 => { self.regs.h = self.regs.c; Event::Normal(5) }
            0x62 => { self.regs.h = self.regs.d; Event::Normal(5) }
            0x63 => { self.regs.h = self.regs.e; Event::Normal(5) }
            0x64 => { self.regs.h = self.regs.h; Event::Normal(5) }
            0x65 => { self.regs.h = self.regs.l; Event::Normal(5) }
            0x66 => { self.regs.h = self.get_m(); Event::Normal(7) }
            0x67 => { self.regs.h = self.regs.a; Event::Normal(5) }

            // MOV L regsm
            0x68 => { self.regs.l = self.regs.b; Event::Normal(5) }
            0x69 => { self.regs.l = self.regs.c; Event::Normal(5) }
            0x6a => { self.regs.l = self.regs.d; Event::Normal(5) }
            0x6b => { self.regs.l = self.regs.e; Event::Normal(5) }
            0x6c => { self.regs.l = self.regs.h; Event::Normal(5) }
            0x6d => { self.regs.l = self.regs.l; Event::Normal(5) }
            0x6e => { self.regs.l = self.get_m(); Event::Normal(7) }
            0x6f => { self.regs.l = self.regs.a; Event::Normal(5) }

            // MOV M regs
            0x70 => { self.set_m(self.regs.b); Event::Normal(7) }
            0x71 => { self.set_m(self.regs.c); Event::Normal(7) }
            0x72 => { self.set_m(self.regs.d); Event::Normal(7) }
            0x73 => { self.set_m(self.regs.e); Event::Normal(7) }
            0x74 => { self.set_m(self.regs.h); Event::Normal(7) }
            0x75 => { self.set_m(self.regs.l); Event::Normal(7) }
            0x77 => { self.set_m(self.regs.a); Event::Normal(7) }

            // MOV A regsm
            0x78 => { self.regs.a = self.regs.b; Event::Normal(5) }
            0x79 => { self.regs.a = self.regs.c; Event::Normal(5) }
            0x7a => { self.regs.a = self.regs.d; Event::Normal(5) }
            0x7b => { self.regs.a = self.regs.e; Event::Normal(5) }
            0x7c => { self.regs.a = self.regs.h; Event::Normal(5) }
            0x7d => { self.regs.a = self.regs.l; Event::Normal(5) }
            0x7e => { self.regs.a = self.get_m(); Event::Normal(7) }
            0x7f => { self.regs.a = self.regs.a; Event::Normal(5) }

            // MVI
            0x06 => {
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.b = data;
                Event::Normal(7)
            }
            0x0e => {
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.c = data;
                Event::Normal(7)
            }
            0x16 => {
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.d = data;
                Event::Normal(7)
            }
            0x1e => {
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.e = data;
                Event::Normal(7)
            }
            0x26 => {
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.h = data;
                Event::Normal(7)
            }
            0x2e => {
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.l = data;
                Event::Normal(7)
            }
            0x36 => {
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.set_m(data);
                Event::Normal(10)
            }
            0x3e => {
                let data = self.memory.read(self.pc.wrapping_add(1).into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.a = data;
                Event::Normal(7)
            }

            // SHLD
            0x22 => {
                let addr = self.memory.read16(self.pc.into());
                self.pc = self.pc.wrapping_add(2);
                self.memory.write(addr.into(), self.regs.h);
                self.memory.write((addr + 1).into(), self.regs.h);
                Event::Normal(16)
            }

            // STA
            0x32 => {
                let addr = self.memory.read16(self.pc.into());
                self.pc = self.pc.wrapping_add(2);
                self.memory.write(addr.into(), self.regs.a);
                Event::Normal(13)
            }

            // LDAX
            0x0a => {
                let addr = self.regs.get_bc();
                self.memory.write(addr.into(), self.regs.a);
                Event::Normal(7)
            }
            0x1a => {
                let addr = self.regs.get_de();
                self.memory.write(addr.into(), self.regs.a);
                Event::Normal(7)
            }

            // LHLD
            0x2a => {
                let data = self.memory.read16(self.pc.into());
                self.regs.set_hl(data);
                Event::Normal(16)
            }

            // LDA
            0x3a => {
                let addr = self.memory.read16(self.pc.into());
                self.memory.write(addr.into(), self.regs.a);
                Event::Normal(13)
            }

            // JMP
            0xc3 => { self.jmp(true); Event::Normal(13) }
            0xcb => { self.jmp(true); Event::Normal(13) }

            // JC
            0xda => { self.jmp(self.regs.get_flag(Flag::C)); Event::Normal(13) }

            // JNC
            0xd2 => { self.jmp(!self.regs.get_flag(Flag::C)); Event::Normal(13) }

            // JZ
            0xca => { self.jmp(self.regs.get_flag(Flag::Z)); Event::Normal(13) }

            // JNZ
            0xc2 => { self.jmp(!self.regs.get_flag(Flag::Z)); Event::Normal(13) }

            // JP
            0xf2 => { self.jmp(!self.regs.get_flag(Flag::S)); Event::Normal(13) }

            // JM
            0xfa => { self.jmp(self.regs.get_flag(Flag::S)); Event::Normal(13) }

            // JPE
            0xea => { self.jmp(self.regs.get_flag(Flag::P)); Event::Normal(13) }

            // JPO
            0xe2 => { self.jmp(!self.regs.get_flag(Flag::P)); Event::Normal(13) }

            // PCHL
            0xe9 => { self.pc = self.regs.get_hl(); Event::Normal(5) }

            // SPHL
            0xf9 => { self.sp = self.regs.get_hl(); Event::Normal(5) }

            // XTHL
            0xe3 => {
                let data = self.memory.read16(self.sp.into());
                self.sp = self.regs.get_hl();
                self.regs.set_hl(data);
                Event::Normal(18)
            }

            // XCHG
            0xeb => {
                let tmp = self.regs.get_hl();
                self.regs.set_hl(self.regs.get_de());
                self.regs.set_de(tmp);
                Event::Normal(5)
            }

            // CALL
            0xcd => self.call(true),
            0xdd => self.call(true),
            0xed => self.call(true),
            0xfd => self.call(true),

            // CC
            0xdc => self.call(self.regs.get_flag(Flag::C)),

            // CNC
            0xd4 => self.call(!self.regs.get_flag(Flag::C)),

            // CZ
            0xcc => self.call(self.regs.get_flag(Flag::Z)),

            // CNZ
            0xc4 => self.call(!self.regs.get_flag(Flag::Z)),

            // CP
            0xf4 => self.call(!self.regs.get_flag(Flag::S)),

            // CM
            0xfc => self.call(self.regs.get_flag(Flag::S)),

            // CPE
            0xec => self.call(self.regs.get_flag(Flag::P)),

            // CPO
            0xe4 => self.call(!self.regs.get_flag(Flag::P)),

            // RET
            0xc9 => self.ret(true),
            0xd9 => self.ret(true),

            // RC
            0xd8 => self.ret(self.regs.get_flag(Flag::C)),

            // RNC
            0xd0 => self.ret(!self.regs.get_flag(Flag::C)),

            // RZ
            0xc8 => self.ret(self.regs.get_flag(Flag::Z)),

            // RNZ
            0xc0 => self.ret(!self.regs.get_flag(Flag::Z)),

            // RM
            0xf8 => self.ret(self.regs.get_flag(Flag::S)),

            // RP
            0xf0 => self.ret(!self.regs.get_flag(Flag::S)),

            // RPE
            0xe8 => self.ret(self.regs.get_flag(Flag::P)),

            // RPO
            0xe0 => self.ret(!self.regs.get_flag(Flag::P)),

            // PUSH
            0xc5 => { self.push(self.regs.get_bc()); Event::Normal(11) }
            0xd5 => { self.push(self.regs.get_de()); Event::Normal(11) }
            0xe5 => { self.push(self.regs.get_hl()); Event::Normal(11) }
            0xf5 => { self.push(self.regs.get_af()); Event::Normal(11) }

            // POP
            0xc1 => { let data = self.pop(); self.regs.set_bc(data); Event::Normal(10) }
            0xd1 => { let data = self.pop(); self.regs.set_de(data); Event::Normal(10) }
            0xe1 => { let data = self.pop(); self.regs.set_hl(data); Event::Normal(10) }
            0xf1 => { let data = self.pop(); self.regs.set_af(data); Event::Normal(10) }

            // EI
            0xfb => { self.inte = true; Event::Normal(4) }
            // DI
            0xf3 => { self.inte = false; Event::Normal(4) }

            // IN
            0xdb => { 
                let data = self.memory.read(self.pc.into());
                println!("Read byte from input device: {}", data);
                Event::Normal(10)
            }

            // OUT
            0xd3 => {
                let data = self.memory.read(self.pc.into());
                println!("Send byte to input device: {}", data);
                Event::Output(data, self.regs.a, 10)
            }

            // HLT
            0x76 => Event::Halt(7),

            // RST
            0xc7 => { self.rst(0b0000_0000_0000_0000); Event::Normal(11) }
            0xcf => { self.rst(0b0000_0000_0000_1000); Event::Normal(11) }
            0xd7 => { self.rst(0b0000_0000_0001_0000); Event::Normal(11) }
            0xdf => { self.rst(0b0000_0000_0001_1000); Event::Normal(11) }
            0xe7 => { self.rst(0b0000_0000_0010_0000); Event::Normal(11) }
            0xef => { self.rst(0b0000_0000_0010_1000); Event::Normal(11) }
            0xf7 => { self.rst(0b0000_0000_0011_0000); Event::Normal(11) }
            0xff => { self.rst(0b0000_0000_0011_1000); Event::Normal(11) }

            // _ => panic!("Instruction not implemented: {:x}", op),
        }
    }
}
