use crate::{hexdump, mem::Memory, reg::Registers};
use log::debug;

#[derive(Default)]
pub struct Cpu {
    reg: Registers,
    mem: Memory,
}

impl Cpu {
    pub fn new(mem: Memory) -> Self {
        Self {
            mem,
            reg: Registers::default(),
        }
    }
    pub fn set_pc(&mut self, pc: u16) {
        self.reg.pc = pc;
    }
    pub fn get_mem(&self) -> &Memory {
        &self.mem
    }
    pub fn dump_mem(&self) {
        hexdump::dump(&self.get_mem().get());
    }
}

impl Cpu {
    fn dispatch_opcode(&mut self, opc: u8) -> Option<i32> {
        let size = match opc {
            0x18 => {
                debug!("CLC");
                self.reg.sr.c = false;
                // reg.sr = reg.sr & !FL_C;
                1
            }
            0x20 => {
                let ret = self.reg.pc + 2;
                self.mem.store16(self.reg.sp as u16 + 0x100, ret);
                self.reg.sp -= 2;
                self.reg.pc = self.mem.load16(self.reg.pc + 1);
                debug!("JSR -> {:x} {:x}", self.reg.pc, ret);
                0
            }
            0x25 => {
                self.reg.and(self.load_zeropage());
                2
            }
            0x29 => {
                self.reg.and(self.load_immediate());
                2
            }
            0x2d => {
                self.reg.and(self.load_absolute());
                3
            }
            0x4c => {
                self.reg.pc = self.mem.load16(self.reg.pc + 1);
                0
            }
            0x60 => {
                self.reg.sp += 2;
                self.reg.pc = self.mem.load16(self.reg.sp as u16 + 0x100);
                debug!("RTS -> {:x}", self.reg.pc);
                1
            }
            0x65 => {
                self.reg.adc(self.load_zeropage());
                2
            }
            0x69 => {
                self.reg.adc(self.load_immediate());
                2
            }
            0x6d => {
                self.reg.adc(self.load_absolute());
                3
            }
            0x85 => {
                // debug!("STA");
                // self.mem.store(self.mem.load16(self.reg.pc + 1), self.reg.a);
                self.store_zeropage(self.reg.a);
                2
            }
            0x8d => {
                debug!("STA");
                // self.mem.store(self.mem.load16(self.reg.pc + 1), self.reg.a);
                self.store_absolute(self.reg.a);
                3
            }
            // LDA
            0xa9 => {
                self.reg.lda(self.load_immediate());
                2
            }
            0xa5 => {
                self.reg.lda(self.load_zeropage());
                2
            }
            0xb5 => {
                self.reg.lda(self.load_zeropage_x());
                2
            }
            0xad => {
                self.reg.lda(self.load_absolute());
                3
            }
            0xbd => {
                self.reg.lda(self.load_absolute_x());
                3
            }
            0xb9 => {
                self.reg.lda(self.load_absolute_y());
                3
            }
            // LDY
            0xa4 => {
                self.reg.ldy(self.load_zeropage());
                2
            }
            // LDX
            0xa6 => {
                self.reg.ldx(self.load_zeropage());
                2
            }
            0xaa => {
                debug!("TAX");
                self.reg.x = self.reg.a;
                self.reg.sr.update_nz(self.reg.x);
                1
            }
            0xe8 => {
                debug!("INX");
                let res = (self.reg.x as u16).wrapping_add(1);
                self.reg.sr.update_nvzc(res);
                self.reg.x = res as u8;
                1
            }
            0 => {
                println!("break on 00");
                return None;
            }
            _ => panic!("unhandled opcode: {:x} pc: {:x}", opc, self.reg.pc),
        };
        Some(size)
    }
}

impl Cpu {
    fn load_zeropage(&self) -> u8 {
        let zp_addr = self.mem.load(self.reg.pc + 1);
        let oper = self.mem.load(zp_addr as u16);
        oper
    }
    fn load_zeropage_x(&self) -> u8 {
        let zp_addr = self.mem.load(self.reg.pc + 1);
        let addr = zp_addr.wrapping_add(self.reg.x);
        let oper = self.mem.load(addr as u16);
        oper
    }
    fn load_absolute(&self) -> u8 {
        let addr = self.mem.load16(self.reg.pc + 1);
        let oper = self.mem.load(addr);
        oper
    }
    fn load_absolute_x(&self) -> u8 {
        let addr = self.mem.load16(self.reg.pc + 1);
        let addr = addr
            .wrapping_add(self.reg.x as u16)
            .wrapping_add(self.reg.sr.carry());
        let oper = self.mem.load(addr);
        oper
    }
    fn load_absolute_y(&self) -> u8 {
        let addr = self.mem.load16(self.reg.pc + 1);
        let addr = addr
            .wrapping_add(self.reg.y as u16)
            .wrapping_add(self.reg.sr.carry());
        let oper = self.mem.load(addr);
        oper
    }
    fn load_immediate(&self) -> u8 {
        let oper = self.mem.load(self.reg.pc + 1);
        oper
    }
    fn store_zeropage(&mut self, v: u8) {
        self.mem.store(self.mem.load(self.reg.pc + 1) as u16, v);
    }
    fn store_absolute(&mut self, v: u8) {
        self.mem.store(self.mem.load16(self.reg.pc + 1), v);
    }
    pub fn run(&mut self) {
        // let reg = &mut self.reg;
        // let mem = &mut self.mem;
        loop {
            let opc = self.mem.load(self.reg.pc);
            debug!(
                "pc: {:03x}, opc: {:02x}, reg: {}",
                self.reg.pc, opc, self.reg
            );
            let size = match self.dispatch_opcode(opc) {
                Some(value) => value,
                None => break,
            };
            self.reg.pc += size as u16;
        }
    }
}
