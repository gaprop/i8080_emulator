use std::collections::HashMap;
use crate::memory::{Memory};

#[derive(Debug)]
enum Opcode {
    SingleOpcode(&'static str),
    Immediate8(&'static str),
    Immediate16(&'static str),
    DirectAdress(&'static str),

    RegPairFirstOperand(&'static str, &'static str),
    RegPairSecOperand(&'static str),
    RegPairAndImm(&'static str),
}

pub struct Disassembler {
    ins: HashMap<u8, Opcode>
}

impl Disassembler {
    pub fn disassemble(&self, memory: &impl Memory, pc: &u16, op: &u8, rp: &u16) -> String {
        let code = self.ins.get(op);
        match code {
            Some(Opcode::SingleOpcode(n)) => {
                format!("{:x}    {}", pc, n)
            }
            Some(Opcode::Immediate8(n)) => {
                let imm8 = memory.read((pc + 1).into());
                format!("{:x}    {} 0x{:x}", pc, n, imm8)
            }
            Some(Opcode::Immediate16(n)) => {
                let imm16 = memory.read16((pc + 1).into());
                format!("{:x}    {} 0x{:x}", pc, n, imm16)
            }
            Some(Opcode::DirectAdress(n)) => {
                let imm16 = memory.read16((pc + 1).into());
                format!("{:x}    {} $(0x{:x})", pc, n, imm16)
            }
            Some(Opcode::RegPairFirstOperand(n1, n2)) => {
                format!("{:x}    {} $(0x{:x}), {}", pc, n1, rp, n2)
            }
            Some(Opcode::RegPairSecOperand(n)) => {
                format!("{:x}    {} $(0x{:x})", pc, n, rp)
            }
            Some(Opcode::RegPairAndImm(n)) => {
                let imm8 = memory.read((pc + 1).into());
                format!("{:x}    {} $(0x{:x}), 0x{:x}", pc, n, rp, imm8)
            }
            n => panic!("Could not disassemble: {:#?}", n),
        }
    }
    pub fn new() -> Self {
        let opcodes = vec![
            (0x00, Opcode::SingleOpcode("NOP")),
            (0x10, Opcode::SingleOpcode("NOP")),
            (0x20, Opcode::SingleOpcode("NOP")),
            (0x30, Opcode::SingleOpcode("NOP")),
            (0x08, Opcode::SingleOpcode("NOP")),
            (0x18, Opcode::SingleOpcode("NOP")),
            (0x28, Opcode::SingleOpcode("NOP")),
            (0x38, Opcode::SingleOpcode("NOP")),


            (0x07, Opcode::SingleOpcode("RLC")),
            (0x17, Opcode::SingleOpcode("RAL")),
            (0x0f, Opcode::SingleOpcode("RRC")),
            (0x1f, Opcode::SingleOpcode("RAR")),

            (0x27, Opcode::SingleOpcode("DAA")),
            (0x37, Opcode::SingleOpcode("STC")),
            (0x2f, Opcode::SingleOpcode("CMA")),
            (0x3f, Opcode::SingleOpcode("CMC")),
            (0xe3, Opcode::SingleOpcode("XTHL")),
            (0xf3, Opcode::SingleOpcode("DI")),

            (0xc9, Opcode::SingleOpcode("RET")),
            (0xd9, Opcode::SingleOpcode("RET")),
            (0xc8, Opcode::SingleOpcode("RZ")),
            (0xd8, Opcode::SingleOpcode("RC")),
            (0xe8, Opcode::SingleOpcode("RPE")),
            (0xf8, Opcode::SingleOpcode("RM")),
            (0xc0, Opcode::SingleOpcode("RNZ")),
            (0xd0, Opcode::SingleOpcode("RNC")),
            (0xe0, Opcode::SingleOpcode("RPO")),
            (0xf0, Opcode::SingleOpcode("RP")),

            (0xc2, Opcode::DirectAdress("JNZ")),
            (0xc3, Opcode::DirectAdress("JMP")),
            (0xca, Opcode::DirectAdress("JZ")),
            (0xcb, Opcode::DirectAdress("JMP")),
            (0xd2, Opcode::DirectAdress("JNC")),
            (0xda, Opcode::DirectAdress("JC")),
            (0xe2, Opcode::DirectAdress("JPO")),
            (0xea, Opcode::DirectAdress("JPE")),
            (0xf2, Opcode::DirectAdress("JP")),
            (0xfa, Opcode::DirectAdress("JM")),

            (0xc4, Opcode::DirectAdress("CNZ")),
            (0xcc, Opcode::DirectAdress("CZ")),
            (0xcd, Opcode::DirectAdress("CALL")),
            (0xd4, Opcode::DirectAdress("CNC")),
            (0xdc, Opcode::DirectAdress("CC")),
            (0xdd, Opcode::DirectAdress("CALL")),
            (0xe4, Opcode::DirectAdress("CPO")),
            (0xec, Opcode::DirectAdress("CPE")),
            (0xed, Opcode::DirectAdress("CALL")),
            (0xf4, Opcode::DirectAdress("CP")),
            (0xfc, Opcode::DirectAdress("CM")),
            (0xfd, Opcode::DirectAdress("CALL")),

            (0xe9, Opcode::SingleOpcode("PCHL")),
            (0xf9, Opcode::SingleOpcode("SPHL")),
            (0xeb, Opcode::SingleOpcode("XCHG")),
            (0xfb, Opcode::SingleOpcode("EI")),

            (0xc7, Opcode::SingleOpcode("RST 0")),
            (0xcf, Opcode::SingleOpcode("RST 1")),
            (0xd7, Opcode::SingleOpcode("RST 2")),
            (0xdf, Opcode::SingleOpcode("RST 3")),
            (0xe7, Opcode::SingleOpcode("RST 4")),
            (0xef, Opcode::SingleOpcode("RST 5")),
            (0xf7, Opcode::SingleOpcode("RST 6")),
            (0xff, Opcode::SingleOpcode("RST 7")),
              
            // Is this a double register?
            (0x02, Opcode::SingleOpcode("STAX B")),
            (0x12, Opcode::SingleOpcode("STAX D")),

            (0x22, Opcode::DirectAdress("SHLD")),
            (0x2a, Opcode::DirectAdress("LHLD")),
            (0x32, Opcode::DirectAdress("STA")),
            (0x3a, Opcode::DirectAdress("LDA")),

            // Is this a double register?
            (0x03, Opcode::SingleOpcode("INX BC")),
            (0x13, Opcode::SingleOpcode("INX DE")),
            (0x23, Opcode::SingleOpcode("INX HL")),
            (0x33, Opcode::SingleOpcode("INX SP")),

            // Is this a double register?
            (0x04, Opcode::SingleOpcode("INR B")),
            (0x14, Opcode::SingleOpcode("INR D")),
            (0x24, Opcode::SingleOpcode("INR H")),
            (0x34, Opcode::RegPairSecOperand("INR")),
            (0x0c, Opcode::SingleOpcode("INR C")),
            (0x1c, Opcode::SingleOpcode("INR E")),
            (0x2c, Opcode::SingleOpcode("INR L")),
            (0x3c, Opcode::SingleOpcode("INR A")),

            // Is this a double register?
            (0x05, Opcode::SingleOpcode("DCR B")),
            (0x15, Opcode::SingleOpcode("DCR D")),
            (0x25, Opcode::SingleOpcode("DCR H")),
            (0x35, Opcode::RegPairSecOperand("DCR")),
            (0x0d, Opcode::SingleOpcode("DCR C")),
            (0x1d, Opcode::SingleOpcode("DCR E")),
            (0x2d, Opcode::SingleOpcode("DCR L")),
            (0x3d, Opcode::SingleOpcode("DCR A")),

            // Is this a double register?
            (0x09, Opcode::SingleOpcode("DAD B")),
            (0x19, Opcode::SingleOpcode("DAD D")),
            (0x29, Opcode::SingleOpcode("DAD H")),
            (0x39, Opcode::SingleOpcode("DAD SP")),

            // Is this a double register?
            (0x0a, Opcode::SingleOpcode("LDAX B")),
            (0x1a, Opcode::SingleOpcode("LDAX D")),

            // Is this a double register?
            (0x0b, Opcode::SingleOpcode("DCX B")),
            (0x1b, Opcode::SingleOpcode("DCX D")),
            (0x2b, Opcode::SingleOpcode("DCX H")),
            (0x3b, Opcode::SingleOpcode("DCX SP")),

            // Is this a double register?
            (0x01, Opcode::Immediate16("LXI BC")),
            (0x11, Opcode::Immediate16("LXI DE")),
            (0x21, Opcode::Immediate16("LXI HL")),
            (0x31, Opcode::Immediate16("LXI SP")),

            (0x40, Opcode::SingleOpcode("MOV B, B")),
            (0x50, Opcode::SingleOpcode("MOV D, B")),
            (0x60, Opcode::SingleOpcode("MOV H, B")),
            (0x70, Opcode::RegPairFirstOperand("MOV", "B")),

            (0x41, Opcode::SingleOpcode("MOV B, C")),
            (0x51, Opcode::SingleOpcode("MOV D, C")),
            (0x61, Opcode::SingleOpcode("MOV H, C")),
            (0x71, Opcode::RegPairFirstOperand("MOV", "C")),

            (0x42, Opcode::SingleOpcode("MOV B, D")),
            (0x52, Opcode::SingleOpcode("MOV D, D")),
            (0x62, Opcode::SingleOpcode("MOV H, D")),
            (0x72, Opcode::RegPairFirstOperand("MOV", "D")),

            (0x43, Opcode::SingleOpcode("MOV B, E")),
            (0x53, Opcode::SingleOpcode("MOV D, E")),
            (0x63, Opcode::SingleOpcode("MOV H, E")),
            (0x73, Opcode::RegPairFirstOperand("MOV", "E")),

            (0x44, Opcode::SingleOpcode("MOV B, H")),
            (0x54, Opcode::SingleOpcode("MOV D, H")),
            (0x64, Opcode::SingleOpcode("MOV H, H")),
            (0x74, Opcode::RegPairFirstOperand("MOV", "H")),

            (0x45, Opcode::SingleOpcode("MOV B, L")),
            (0x55, Opcode::SingleOpcode("MOV D, L")),
            (0x65, Opcode::SingleOpcode("MOV H, L")),

            (0x46, Opcode::RegPairSecOperand("MOV C")),
            (0x56, Opcode::RegPairSecOperand("MOV E")),
            (0x66, Opcode::RegPairSecOperand("MOV L")),
            (0x76, Opcode::RegPairSecOperand("MOV A")),

            (0x47, Opcode::SingleOpcode("MOV B, A")),
            (0x57, Opcode::SingleOpcode("MOV D, A")),
            (0x67, Opcode::SingleOpcode("MOV H, A")),
            (0x77, Opcode::RegPairFirstOperand("MOV", "A")),

            (0x48, Opcode::SingleOpcode("MOV C, B")),
            (0x58, Opcode::SingleOpcode("MOV E, B")),
            (0x68, Opcode::SingleOpcode("MOV L, B")),
            (0x78, Opcode::SingleOpcode("MOV A, B")),

            (0x49, Opcode::SingleOpcode("MOV C, C")),
            (0x59, Opcode::SingleOpcode("MOV E, C")),
            (0x69, Opcode::SingleOpcode("MOV L, C")),
            (0x79, Opcode::SingleOpcode("MOV A, C")),

            (0x4a, Opcode::SingleOpcode("MOV C, D")),
            (0x5a, Opcode::SingleOpcode("MOV E, D")),
            (0x6a, Opcode::SingleOpcode("MOV L, D")),
            (0x7a, Opcode::SingleOpcode("MOV A, D")),

            (0x4b, Opcode::SingleOpcode("MOV C, E")),
            (0x5b, Opcode::SingleOpcode("MOV E, E")),
            (0x6b, Opcode::SingleOpcode("MOV L, E")),
            (0x7b, Opcode::SingleOpcode("MOV A, E")),

            (0x4c, Opcode::SingleOpcode("MOV C, H")),
            (0x5c, Opcode::SingleOpcode("MOV E, H")),
            (0x6c, Opcode::SingleOpcode("MOV L, H")),
            (0x7c, Opcode::SingleOpcode("MOV A, H")),

            (0x4d, Opcode::SingleOpcode("MOV C, L")),
            (0x5d, Opcode::SingleOpcode("MOV E, L")),
            (0x6d, Opcode::SingleOpcode("MOV L, L")),
            (0x7d, Opcode::SingleOpcode("MOV A, L")),

            (0x4e, Opcode::DirectAdress("MOV C")),
            (0x5e, Opcode::DirectAdress("MOV E")),
            (0x6e, Opcode::DirectAdress("MOV L")),
            (0x7e, Opcode::DirectAdress("MOV A")),

            (0x4f, Opcode::SingleOpcode("MOV C, A")),
            (0x5f, Opcode::SingleOpcode("MOV E, A")),
            (0x6f, Opcode::SingleOpcode("MOV L, A")),
            (0x7f, Opcode::SingleOpcode("MOV A, A")),

            (0x06, Opcode::Immediate8("MVI B")),
            (0x0e, Opcode::Immediate8("MVI C")),
            (0x16, Opcode::Immediate8("MVI D")),
            (0x1e, Opcode::Immediate8("MVI E")),
            (0x26, Opcode::Immediate8("MVI H")),
            (0x2e, Opcode::Immediate8("MVI L")),
            (0x36, Opcode::RegPairAndImm("MVI")),
            (0x3e, Opcode::Immediate8("MVI A")),



            (0x80, Opcode::SingleOpcode("ADD B")),
            (0x81, Opcode::SingleOpcode("ADD C")),
            (0x82, Opcode::SingleOpcode("ADD D")),
            (0x83, Opcode::SingleOpcode("ADD E")),
            (0x84, Opcode::SingleOpcode("ADD H")),
            (0x85, Opcode::SingleOpcode("ADD L")),
            (0x86, Opcode::RegPairSecOperand("ADD")),
            (0x87, Opcode::SingleOpcode("ADD A")),

            (0xc6, Opcode::Immediate8("ADI")),

            (0x88, Opcode::SingleOpcode("ADC B")),
            (0x89, Opcode::SingleOpcode("ADC C")),
            (0x8a, Opcode::SingleOpcode("ADC D")),
            (0x8b, Opcode::SingleOpcode("ADC E")),
            (0x8c, Opcode::SingleOpcode("ADC H")),
            (0x8d, Opcode::SingleOpcode("ADC L")),
            (0x8e, Opcode::RegPairSecOperand("ADC")),
            (0x8f, Opcode::SingleOpcode("ADC A")),

            (0xce, Opcode::Immediate8("ACI")),

            (0x90, Opcode::SingleOpcode("SUB B")),
            (0x91, Opcode::SingleOpcode("SUB C")),
            (0x92, Opcode::SingleOpcode("SUB D")),
            (0x93, Opcode::SingleOpcode("SUB E")),
            (0x94, Opcode::SingleOpcode("SUB H")),
            (0x95, Opcode::SingleOpcode("SUB L")),
            (0x96, Opcode::RegPairSecOperand("SUB")),
            (0x97, Opcode::SingleOpcode("SUB A")),

            (0xd6, Opcode::Immediate8("SUI")),

            (0x98, Opcode::SingleOpcode("SBB B")),
            (0x99, Opcode::SingleOpcode("SBB C")),
            (0x9a, Opcode::SingleOpcode("SBB D")),
            (0x9b, Opcode::SingleOpcode("SBB E")),
            (0x9c, Opcode::SingleOpcode("SBB H")),
            (0x9d, Opcode::SingleOpcode("SBB L")),
            (0x9e, Opcode::RegPairSecOperand("SBB")),
            (0x9f, Opcode::SingleOpcode("SBB A")),

            (0xde, Opcode::Immediate8("SBI")),

            (0xa0, Opcode::SingleOpcode("ANA B")),
            (0xa1, Opcode::SingleOpcode("ANA C")),
            (0xa2, Opcode::SingleOpcode("ANA D")),
            (0xa3, Opcode::SingleOpcode("ANA E")),
            (0xa4, Opcode::SingleOpcode("ANA H")),
            (0xa5, Opcode::SingleOpcode("ANA L")),
            (0xa6, Opcode::RegPairSecOperand("ANA")),
            (0xa7, Opcode::SingleOpcode("ANA A")),

            (0xe6, Opcode::Immediate8("ANI")),

            (0xa8, Opcode::SingleOpcode("XRA B")),
            (0xa9, Opcode::SingleOpcode("XRA C")),
            (0xaa, Opcode::SingleOpcode("XRA D")),
            (0xab, Opcode::SingleOpcode("XRA E")),
            (0xac, Opcode::SingleOpcode("XRA H")),
            (0xad, Opcode::SingleOpcode("XRA L")),
            (0xae, Opcode::RegPairSecOperand("XRA")),
            (0xaf, Opcode::SingleOpcode("XRA A")),

            (0xee, Opcode::Immediate8("XRI")),

            (0xb0, Opcode::SingleOpcode("ORA B")),
            (0xb1, Opcode::SingleOpcode("ORA C")),
            (0xb2, Opcode::SingleOpcode("ORA D")),
            (0xb3, Opcode::SingleOpcode("ORA E")),
            (0xb4, Opcode::SingleOpcode("ORA H")),
            (0xb5, Opcode::SingleOpcode("ORA L")),
            (0xb6, Opcode::RegPairSecOperand("ORA")),
            (0xb7, Opcode::SingleOpcode("ORA A")),

            (0xf6, Opcode::Immediate8("ORI")),

            (0xb8, Opcode::SingleOpcode("CMP B")),
            (0xb9, Opcode::SingleOpcode("CMP C")),
            (0xba, Opcode::SingleOpcode("CMP D")),
            (0xbb, Opcode::SingleOpcode("CMP E")),
            (0xbc, Opcode::SingleOpcode("CMP H")),
            (0xbd, Opcode::SingleOpcode("CMP L")),
            (0xbe, Opcode::RegPairSecOperand("CMP")),
            (0xbf, Opcode::SingleOpcode("CMP A")),

            (0xfe, Opcode::Immediate8("CPI")),

            (0xc1, Opcode::SingleOpcode("POP BC")),
            (0xd1, Opcode::SingleOpcode("POP DE")),
            (0xe1, Opcode::SingleOpcode("POP HL")),
            (0xf1, Opcode::SingleOpcode("POP PSW")),

            (0xc5, Opcode::SingleOpcode("PUSH BC")),
            (0xd5, Opcode::SingleOpcode("PUSH DE")),
            (0xe5, Opcode::SingleOpcode("PUSH HL")),
            (0xf5, Opcode::SingleOpcode("PUSH PSW")),

            (0xd3, Opcode::Immediate8("OUT")),
            (0xdb, Opcode::Immediate8("IN")),
        ];
        Disassembler {
            ins: opcodes.into_iter().collect(),
        }
    }
}
