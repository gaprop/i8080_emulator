use crate::memory::{Memory, Memory8080};
use crate::registers::{Registers, Flag};
use crate::device::Device;
use crate::disassembler::{Disassembler};

type ClockCycles = u32;
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
    inter: bool,
    disassembler: Disassembler,
}

impl CPU {
    pub fn new_empty() -> Self {
        CPU {
            regs: Registers::new(),
            memory: Memory8080::new_empty(),
            pc: 0,
            sp: 0x0000, // 0xf000,
            inter: false,
            disassembler: Disassembler::new(),
        }
    }

    pub fn new(memory: [u8; 0x10000]) -> Self {
        CPU {
            regs: Registers::new(),
            memory: Memory8080::new(memory),
            pc: 0,
            sp: 0x0000, // 0xf000,
            inter: false,
            disassembler: Disassembler::new(),
        }
    }

    pub fn inter_handle(&mut self, addr: u16) -> Option<Event> {
        if self.inter {
            self.inter = false;
            self.push(self.pc);
            self.pc = addr;
            return Some(Event::Normal(17));
        }
        None
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
    fn inr(&mut self, n: u8) -> u8 {
        let r = n.wrapping_add(1);
        self.regs.set_flag(Flag::S, (r & 0x80) == 0x80);
        self.regs.set_flag(Flag::Z, r == 0x00);
        self.regs.set_flag(Flag::A, (n & 0x0f) + 1 > 0x0f);
        self.regs.set_flag(Flag::P, r.count_ones() & 0x01 == 0x00);
        r
    }

    fn dcr(&mut self, n: u8) -> u8 {
        let r = n.wrapping_sub(1);
        self.regs.set_flag(Flag::S, (r & 0x80) == 0x80);
        self.regs.set_flag(Flag::Z, r == 0x00);
        self.regs.set_flag(Flag::A, (r & 0x0f) != 0x0f);
        self.regs.set_flag(Flag::P, r.count_ones() & 0x01 == 0x00);
        r
    }

    fn add(&mut self, regm1: u8, regm2: u8) -> u8 {
        let n = regm1.wrapping_add(regm2);
        self.regs.set_flag(Flag::S, (n & 0x80) == 0x80);
        self.regs.set_flag(Flag::Z, n == 0x00);
        self.regs.set_flag(Flag::A, (regm1 & 0x0f) + (regm2 & 0x0f) > 0x0f);
        self.regs.set_flag(Flag::P, n.count_ones() & 0x01 == 0x00);
        self.regs.set_flag(Flag::C, u16::from(regm1) + u16::from(regm2) > 0xff);
        n
    }

    fn adc(&mut self, regm1: u8, regm2: u8) -> u8 {
        let carry = if self.regs.get_flag(Flag::C) { 1 } else { 0 };

        let n = regm1.wrapping_add(regm2).wrapping_add(carry);
        self.regs.set_flag(Flag::S, (n & 0x80) == 0x80);
        self.regs.set_flag(Flag::Z, n == 0x00);
        self.regs.set_flag(Flag::A, (regm1 & 0x0f) + (regm2 & 0x0f) + carry > 0x0f);
        self.regs.set_flag(Flag::P, n.count_ones() & 0x01 == 0x00);
        self.regs.set_flag(Flag::C, u16::from(regm1) + u16::from(regm2) + u16::from(carry) > 0xff);
        n
    }

    fn sub(&mut self, regm1: u8, regm2: u8) -> u8 {
        let n = regm1.wrapping_sub(regm2);
        self.regs.set_flag(Flag::S, (n & 0x80) == 0x80);
        self.regs.set_flag(Flag::Z, n == 0x00);
        self.regs.set_flag(Flag::A, (regm1 as i8 & 0x0f) - (regm2 as i8 & 0x0f) >= 0x00);
        self.regs.set_flag(Flag::P, n.count_ones() & 0x01 == 0x00);
        self.regs.set_flag(Flag::C, u16::from(regm1) < u16::from(regm2));
        n
    }

    fn sbb(&mut self, regm1: u8, regm2: u8) -> u8 {
        let carry = if self.regs.get_flag(Flag::C) { 1 } else { 0 };

        let n = regm1.wrapping_sub(regm2).wrapping_sub(carry);
        self.regs.set_flag(Flag::S, (n & 0x80) == 0x80);
        self.regs.set_flag(Flag::Z, n == 0x00);
        self.regs.set_flag(Flag::A, (regm1 as i8 & 0x0f) - (regm2 as i8 & 0x0f) - (carry as i8 & 0x0f) >= 0x00);
        self.regs.set_flag(Flag::P, n.count_ones() & 0x01 == 0x00);
        self.regs.set_flag(Flag::C, u16::from(regm1) < u16::from(regm2) + u16::from(carry));
        n
    }

    // Bitwise instructions
    fn ana(&mut self, regm1: u8, regm2: u8) -> u8 {
        let n = regm1 & regm2;
        self.regs.set_flag(Flag::S, (n & 0x80) == 0x80);
        self.regs.set_flag(Flag::Z, n == 0x00);
        self.regs.set_flag(Flag::A, ((regm1 | regm2) & 0x08) != 0);
        self.regs.set_flag(Flag::P, n.count_ones() & 0x01 == 0x00);
        self.regs.set_flag(Flag::C, false);
        n
    }

    fn xra(&mut self, regm1: u8, regm2: u8) -> u8 {
        let n = regm1 ^ regm2;
        self.regs.set_flag(Flag::S, (n & 0x80) == 0x80);
        self.regs.set_flag(Flag::Z, n == 0x00);
        self.regs.set_flag(Flag::A, false);
        self.regs.set_flag(Flag::P, n.count_ones() & 0x01 == 0x00);
        self.regs.set_flag(Flag::C, false);
        n
    }

    fn ora(&mut self, regm1: u8, regm2: u8) -> u8 {
        let n = regm1 | regm2;
        self.regs.set_flag(Flag::S, (n & 0x80) == 0x80);
        self.regs.set_flag(Flag::Z, n == 0x00);
        self.regs.set_flag(Flag::A, false);
        self.regs.set_flag(Flag::P, n.count_ones() & 0x01 == 0x00);
        self.regs.set_flag(Flag::C, false);
        n
    }

    fn cmp(&mut self, regm1: u8, regm2: u8) {
        self.sub(regm1, regm2);
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
            self.push(self.pc.wrapping_add(2));

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
            self.pc = self.pop();

            Event::Normal(11)
        } else {
            Event::Normal(5)
        }
    }

    fn push(&mut self, data: u16) {
        self.sp = self.sp.wrapping_sub(2);
        self.memory.write16(self.sp.into(), data);
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
        // let code = self.disassembler.disassemble(&self.memory, &self.pc, &op, &self.regs.get_hl());
        // println!("{: <25}     pc: {:04x}, sp: {:04x}, a: {:02x}, b: {:02x}, c: {:02x}, d: {:02x}, e: {:02x}, h: {:02x}, l: {:02x}, f: {:02x}", 
                 // code,
                 // self.pc,
                 // self.sp,
                 // self.regs.a,
                 // self.regs.b,
                 // self.regs.c,
                 // self.regs.d,
                 // self.regs.e,
                 // self.regs.h,
                 // self.regs.l,
                 // self.regs.f);
        self.pc = self.pc.wrapping_add(1);
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
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.a = self.add(self.regs.a, data);
                Event::Normal(7)
            }

            // ACI
            0xce => { 
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.a = self.adc(self.regs.a, data);
                Event::Normal(7)
            }

            // SUI
            0xd6 => { 
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.a = self.sub(self.regs.a, data);
                Event::Normal(7)
            }

            // SBI
            0xde => { 
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.a = self.sbb(self.regs.a, data);
                Event::Normal(7)
            }

            // ANI
            0xe6 => {
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.a = self.ana(self.regs.a, data);
                Event::Normal(7)
            }

            // XRI
            0xee => {
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.a = self.xra(self.regs.a, data);
                Event::Normal(7)
            }

            // ORI
            0xf6 => {
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.a = self.ora(self.regs.a, data);
                Event::Normal(7)
            }

            // CPI
            0xfe => {
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.cmp(self.regs.a, data);
                Event::Normal(7)
            }

            // RLC
            0x07 => {
                let carry = (self.regs.a & 0x80) >> 7;
                let n = (self.regs.a << 1) | carry;
                self.regs.set_flag(Flag::C, carry == 1);
                self.regs.a = n;

                Event::Normal(4)
            }

            // RRC
            0x0f => {
                let carry = self.regs.a & 0x01;
                let n = if carry == 1 { 0x80 | (self.regs.a >> 1) } else { self.regs.a >> 1 };
                self.regs.set_flag(Flag::C, carry == 1);
                self.regs.a = n;

                Event::Normal(4)
            }

            // RAL
            0x17 => {
                let carry = (self.regs.a & 0x80) >> 7;
                let n = (self.regs.a << 1) | u8::from(self.regs.get_flag(Flag::C));
                self.regs.set_flag(Flag::C, carry == 1);
                self.regs.a = n;
                Event::Normal(4)
            }

            // RAR
            0x1f => {
                let lo = self.regs.a & 1;
                let carry: u8 = if self.regs.get_flag(Flag::C) { 0x80 } else { 0 };
                self.regs.a = self.regs.a >> 1;
                self.regs.a |= carry;
                self.regs.set_flag(Flag::C, lo == 1);
                Event::Normal(4)
            }

            // CMA
            0x2f => {
                self.regs.a = !self.regs.a;
                Event::Normal(4)
            }

            // CMC
            0x3f => {
                let carry = self.regs.get_flag(Flag::C);
                self.regs.set_flag(Flag::C, !carry);
                Event::Normal(4)
            }

            // DAA
            0x27 => {
                let hi = self.regs.a >> 4;
                let lo = self.regs.a & 0x0f;
                let mut res = 0;
                let mut carry = self.regs.get_flag(Flag::C);
                if lo > 9 || self.regs.get_flag(Flag::A) {
                    res += 0x06;
                }

                if hi > 9 || carry || (hi >= 9 && lo > 9) {
                    res += 0x60;
                    carry = true;
                }
                self.regs.a = self.add(self.regs.a, res);
                self.regs.set_flag(Flag::C, carry);
                Event::Normal(4)
            }

            // STC
            0x37 => { self.regs.set_flag(Flag::C, true); Event::Normal(4) }

            // DAD
            0x09 => { 
                let n = self.regs.get_hl().wrapping_add(self.regs.get_bc());
                self.regs.set_flag(Flag::C, self.regs.get_hl() > 0xffff - self.regs.get_bc());
                self.regs.set_hl(n); 
                Event::Normal(10) 
            }
            0x19 => { 
                let n = self.regs.get_hl().wrapping_add(self.regs.get_de());
                self.regs.set_flag(Flag::C, self.regs.get_hl() > 0xffff - self.regs.get_de());
                self.regs.set_hl(n); 
                Event::Normal(10) 
            }
            0x29 => { 
                let n = self.regs.get_hl().wrapping_add(self.regs.get_hl());
                self.regs.set_flag(Flag::C, self.regs.get_hl() > 0xffff - self.regs.get_hl());
                self.regs.set_hl(n); 
                Event::Normal(10) 
            }
            0x39 => { 
                let n = self.regs.get_hl().wrapping_add(self.sp);
                self.regs.set_flag(Flag::C, self.regs.get_hl() > 0xffff - self.sp);
                self.regs.set_hl(n); 
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
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.b = data;
                Event::Normal(7)
            }
            0x0e => {
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.c = data;
                Event::Normal(7)
            }
            0x16 => {
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.d = data;
                Event::Normal(7)
            }
            0x1e => {
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.e = data;
                Event::Normal(7)
            }
            0x26 => {
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.h = data;
                Event::Normal(7)
            }
            0x2e => {
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.l = data;
                Event::Normal(7)
            }
            0x36 => {
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.set_m(data);
                Event::Normal(10)
            }
            0x3e => {
                let data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                self.regs.a = data;
                Event::Normal(7)
            }

            // SHLD
            0x22 => {
                let addr = self.memory.read16(self.pc.into());
                self.pc = self.pc.wrapping_add(2);
                self.memory.write(addr.into(), self.regs.l);
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
                self.regs.a = self.memory.read(addr.into());
                Event::Normal(7)
            }
            0x1a => {
                let addr = self.regs.get_de();
                self.regs.a = self.memory.read(addr.into());
                Event::Normal(7)
            }

            // LHLD
            0x2a => {
                let addr = self.memory.read16(self.pc.into());
                let data = self.memory.read16(addr.into());
                self.pc = self.pc.wrapping_add(2);
                self.regs.set_hl(data);
                Event::Normal(16)
            }

            // LDA
            0x3a => {
                let addr = self.memory.read16(self.pc.into());
                self.pc = self.pc.wrapping_add(2);
                self.regs.a = self.memory.read(addr.into());
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
                self.memory.write16(self.sp.into(), self.regs.get_hl());
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
            0xfb => { self.inter = true; Event::Normal(4) }
            // DI
            0xf3 => { self.inter = false; Event::Normal(4) }

            // IN
            0xdb => { 
                let _data = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                // println!("Read byte from input device: {}", data);
                Event::Normal(10)
            }

            // OUT
            0xd3 => {
                let port = self.memory.read(self.pc.into());
                self.pc = self.pc.wrapping_add(1);
                // println!("Send byte to input device: {}", port);
                Event::Output(port, self.regs.a, 10)
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

#[cfg(test)]
mod tests {
    use crate::cpu::{CPU};
    use crate::device::{Device};
    use crate::memory::{Memory};
    use crate::registers::{Flag};

    #[test]
    fn test_get_m() {
        let mut cpu = CPU::new_empty();
        cpu.regs.set_hl(1);
        cpu.memory.write(1, 0xff);
        assert_eq!(cpu.get_m(), 0xff);
    }

    #[test]
    fn test_set_m() {
        let mut cpu = CPU::new_empty();
        cpu.regs.set_hl(1);
        cpu.set_m(0xff);
        assert_eq!(cpu.memory.read(1), 0xff);
    }
    #[test]
    fn test_jmp() {
        let mut memory = [0; 0x10000];
        memory[0] = 0xc3;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x02ff);
    }

    #[test]
    fn test_jc() {
        let mut memory = [0; 0x10000];
        memory[0] = 0xda;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_flag(Flag::C, true);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x02ff);
    }

    #[test]
    fn test_jnc_no_jump() {
        let mut memory = [0; 0x10000];
        memory[0] = 0xd2;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_flag(Flag::C, true);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x03);
    }

    #[test]
    fn test_jz_no_jump() {
        let mut memory = [0; 0x10000];
        memory[0] = 0xca;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_flag(Flag::Z, false);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x03);
    }

    #[test]
    fn test_jnz() {
        let mut memory = [0; 0x10000];
        memory[0] = 0xc2;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_flag(Flag::Z, false);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x02ff);
    }

    #[test]
    fn test_jp() {
        let mut memory = [0; 0x10000];
        memory[0] = 0xf2;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_flag(Flag::S, false);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x02ff);
    }

    #[test]
    fn test_jm() {
        let mut memory = [0; 0x10000];
        memory[0] = 0xfa;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_flag(Flag::S, true);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x02ff);
    }

    #[test]
    fn test_jpe() {
        let mut memory = [0; 0x10000];
        memory[0] = 0xea;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_flag(Flag::P, true);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x02ff);
    }

    #[test]
    fn test_jpo() {
        let mut memory = [0; 0x10000];
        memory[0] = 0xe2;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_flag(Flag::P, false);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x02ff);
    }

    #[test]
    fn test_call() {
        let mut memory = [0x00; 0x10000];
        memory[0] = 0xcd;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x02ff);
        assert_eq!(cpu.sp, 0xeffe);
        assert_eq!(cpu.memory.read16(0xeffe), 0x03);
    }

    #[test]
    fn test_cc() {
        let mut memory = [0x00; 0x10000];
        memory[0] = 0xdc;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_flag(Flag::C, true);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x02ff);
        assert_eq!(cpu.sp, 0xeffe);
        assert_eq!(cpu.memory.read16(0xeffe), 0x03);
    }

    #[test]
    fn test_cnc_no_jump() {
        let mut memory = [0x00; 0x10000];
        memory[0] = 0xcd;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_flag(Flag::C, true);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x02ff);
        assert_eq!(cpu.sp, 0xeffe);
        assert_eq!(cpu.memory.read16(0xeffe), 0x03);
    }

    #[test]
    fn test_ret() {
        let mut memory = [0x00; 0x10000];
        memory[0] = 0xcd;
        memory[1] = 0xff;
        memory[2] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_flag(Flag::C, true);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.pc, 0x02ff);
        assert_eq!(cpu.sp, 0xeffe);
        assert_eq!(cpu.memory.read16(0xeffe), 0x03);
    }

    #[test]
    fn test_lxi() {
        let mut memory = [0x00; 0x10000];
        memory[0] = 0x21;
        memory[1] = 0x02;
        memory[2] = 0xff;
        let mut cpu = CPU::new(memory);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.regs.get_hl(), 0xff02);
    }

    #[test]
    fn test_push() {
        let mut memory = [0x00; 0x10000];
        memory[0] = 0xd5;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_de(0xff02);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.memory.read16(0xf000 - 2), 0xff02);
    }

    #[test]
    fn test_pop() {
        let mut memory = [0x00; 0x10000];
        memory[0x00] = 0xd1;
        memory[0xf000 - 2] = 0xff;
        memory[0xf000 - 1] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.sp -= 2;
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.regs.get_de(), 0x02ff);
    }

    #[test]
    fn test_xchg() {
        let mut memory = [0x00; 0x10000];
        memory[0] = 0xeb;
        let mut cpu = CPU::new(memory);
        cpu.regs.set_de(0xff02);
        cpu.regs.set_hl(0x1001);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.regs.get_hl(), 0xff02);
        assert_eq!(cpu.regs.get_de(), 0x1001);
    }

    #[test]
    fn test_mvi() {
        let mut memory = [0x00; 0x10000];
        memory[0] = 0x0e;
        memory[1] = 0xff;
        let mut cpu = CPU::new(memory);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.regs.c, 0xff);
    }

    #[test]
    fn test_ani() {
        let mut memory = [0x00; 0x10000];
        memory[0] = 0xe6;
        memory[1] = 0x00;
        let mut cpu = CPU::new(memory);
        cpu.regs.a = 0xff;
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.regs.a, 0x00);
    }

    #[test]
    fn test_adi() {
        let mut memory = [0x00; 0x10000];
        memory[0] = 0xc6;
        memory[1] = 0x01;
        let mut cpu = CPU::new(memory);
        cpu.regs.a = 0xfe;
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.regs.a, 0xff);
    }

    #[test]
    fn test_cpi() {
        let mut memory = [0x00; 0x10000];
        memory[0] = 0xfe;
        memory[1] = 0x02;
        let mut cpu = CPU::new(memory);
        cpu.regs.a = 0x01;
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.regs.f, 0x97);
    }

    #[test]
    fn test_lda() {
        let mut memory = [0x00; 0x10000];
        memory[0] = 0x3a;
        memory[1] = 0x02;
        memory[2] = 0xff;
        memory[0xff02] = 0xff;
        let mut cpu = CPU::new(memory);
        let op = cpu.fetch();
        cpu.exec(op);
        assert_eq!(cpu.regs.a, 0xff);
    }
}
