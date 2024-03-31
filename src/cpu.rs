use std::collections::HashSet;

use crate::{hexdump, mem::Memory, reg::Registers};
use log::{debug, info};

#[derive(Default)]
pub struct Dbg {
    pc_trace: HashSet<u16>,
}

impl Dbg {
    pub fn step(&mut self, reg: &Registers, mem: &Memory) -> bool {
        if !self.pc_trace.insert(reg.pc) {
            info!("cycle");
            return true;
        } else {
            return false;
        }
    }
}
#[derive(Default)]
pub struct Cpu {
    reg: Registers,
    mem: Memory,
    dbg: Dbg,
}

impl Cpu {
    pub fn new(mem: Memory) -> Self {
        Self {
            mem,
            reg: Registers::default(),
            dbg: Dbg::default(),
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
            // CMP
            0xcd => {
                //
                let oper = self.load_absolute();
                debug!("cmp: {:x} {:x}", oper, self.reg.a);
                let cmp = self.reg.a.wrapping_sub(oper);

                self.reg.sr.update_nz(cmp);
                3
            }
            // BEQ
            0xf0 => {
                if self.reg.sr.z {
                    let offs = self.mem.load(self.reg.pc + 1) as i8;
                    debug!("BEQ -> {}", offs);
                    self.reg.pc = self.reg.pc.wrapping_add_signed(offs.into());
                    2
                } else {
                    debug!("BEQ no");
                    2
                }
            }
            // STA
            0x85 => {
                // debug!("STA");
                // self.mem.store(self.mem.load16(self.reg.pc + 1), self.reg.a);
                self.store_zeropage(self.reg.a);
                2
            }
            0x95 => {
                // debug!("STA");
                // self.mem.store(self.mem.load16(self.reg.pc + 1), self.reg.a);
                self.store_zeropage_x(self.reg.a);
                2
            }
            0x8d => {
                debug!("STA");
                // self.mem.store(self.mem.load16(self.reg.pc + 1), self.reg.a);
                self.store_absolute(self.reg.a);
                3
            }
            0x9d => {
                self.store_absolute_x(self.reg.a);
                3
            }
            0x99 => {
                self.store_absolute_y(self.reg.a);
                3
            }
            0x91 => {
                self.store_indirect_y(self.reg.a);
                2
            }
            0x81 => {
                self.store_indirect_x(self.reg.a);
                2
            }
            // STX
            0x86 => {
                self.store_zeropage(self.reg.x);
                2
            }
            0x96 => {
                self.store_zeropage_y(self.reg.x);
                2
            }
            0x8e => {
                self.store_absolute(self.reg.x);
                3
            }
            // STY
            0x84 => {
                self.store_zeropage(self.reg.y);
                2
            }
            0x94 => {
                self.store_zeropage_x(self.reg.y);
                2
            }

            0x8c => {
                self.store_absolute(self.reg.y);
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
            0xb1 => {
                self.reg.lda(self.load_indirect_y());
                2
            }
            0xa1 => {
                self.reg.lda(self.load_indirect_x());
                2
            }
            // LDY
            0xa0 => {
                self.reg.ldy(self.load_immediate());
                2
            }
            0xa4 => {
                self.reg.ldy(self.load_zeropage());
                2
            }
            0xb4 => {
                self.reg.ldy(self.load_zeropage_x());
                2
            }
            0xac => {
                self.reg.ldy(self.load_absolute());
                3
            }
            0xbc => {
                self.reg.ldy(self.load_absolute_x());
                3
            }
            // LDX
            0xa2 => {
                self.reg.ldx(self.load_immediate());
                2
            }
            0xa6 => {
                self.reg.ldx(self.load_zeropage());
                2
            }
            0xb6 => {
                self.reg.ldx(self.load_zeropage_y());
                2
            }
            0xae => {
                self.reg.ldx(self.load_absolute());
                3
            }
            0xbe => {
                self.reg.ldx(self.load_absolute_y());
                3
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
    fn load_zeropage_y(&self) -> u8 {
        let zp_addr = self.mem.load(self.reg.pc + 1);
        let addr = zp_addr.wrapping_add(self.reg.y);
        let oper = self.mem.load(addr as u16);
        oper
    }
    fn load_absolute(&self) -> u8 {
        let addr = self.mem.load16(self.reg.pc + 1);
        let oper = self.mem.load(addr);
        oper
    }
    fn load_absolute_x(&self) -> u8 {
        let addr = self.addr_absolute_x();
        let oper = self.mem.load(addr);
        oper
    }
    fn load_absolute_y(&self) -> u8 {
        let addr = self.addr_absolute_y();
        let oper = self.mem.load(addr);
        oper
    }
    fn load_immediate(&self) -> u8 {
        let oper = self.mem.load(self.reg.pc + 1);
        oper
    }
    fn load_indirect_y(&self) -> u8 {
        let eff_addr = self.addr_indirect_y();
        self.mem.load(eff_addr)
    }
    fn load_indirect_x(&self) -> u8 {
        let eff_addr = self.addr_indirect_x();
        self.mem.load(eff_addr)
    }
    fn store_zeropage(&mut self, v: u8) {
        self.mem.store(self.mem.load(self.reg.pc + 1) as u16, v);
    }
    fn store_zeropage_x(&mut self, v: u8) {
        let zp_addr = self.mem.load(self.reg.pc + 1);
        let addr = zp_addr.wrapping_add(self.reg.x);
        self.mem.store(addr as u16, v);
    }
    fn store_zeropage_y(&mut self, v: u8) {
        let zp_addr = self.mem.load(self.reg.pc + 1);
        let addr = zp_addr.wrapping_add(self.reg.y);
        self.mem.store(addr as u16, v);
    }
    fn store_absolute(&mut self, v: u8) {
        self.mem.store(self.mem.load16(self.reg.pc + 1), v);
    }
    fn store_absolute_x(&mut self, v: u8) {
        let addr = self.addr_absolute_x();
        self.mem.store(addr, v);
    }

    fn store_absolute_y(&mut self, v: u8) {
        let addr = self.addr_absolute_y();
        self.mem.store(addr, v);
    }

    fn store_indirect_y(&mut self, v: u8) {
        let eff_addr = self.addr_indirect_y();
        self.mem.store(eff_addr, v);
    }

    fn store_indirect_x(&mut self, v: u8) {
        let zp_addr = self.addr_indirect_x();
        self.mem.store(zp_addr as u16, v);
    }

    fn addr_absolute_x(&self) -> u16 {
        let addr = self.mem.load16(self.reg.pc + 1);
        let addr = addr
            .wrapping_add(self.reg.x as u16)
            .wrapping_add(self.reg.sr.carry());
        addr
    }
    fn addr_absolute_y(&self) -> u16 {
        let addr = self.mem.load16(self.reg.pc + 1);
        let addr = addr
            .wrapping_add(self.reg.y as u16)
            .wrapping_add(self.reg.sr.carry());
        addr
    }
    fn addr_indirect_y(&self) -> u16 {
        let zp_addr = self.mem.load(self.reg.pc + 1);
        let addr = self.mem.load16(zp_addr as u16);

        debug!("zp_addr: {:x} {:x}", zp_addr, addr);
        let eff_addr = addr
            .wrapping_add(self.reg.y as u16)
            .wrapping_add(self.reg.sr.carry());
        eff_addr
    }
    fn addr_indirect_x(&self) -> u16 {
        // let zp_addr = self.mem.load(self.reg.pc + 1).wrapping_add(self.reg.x);
        // self.mem.load16(zp_addr as u16)
        let ll = self.mem.load(self.reg.pc + 1);
        let ll = ll.wrapping_add(self.reg.x);
        self.mem.load16(ll as u16)
    }
    pub fn run(&mut self) {
        // let reg = &mut self.reg;
        // let mem = &mut self.mem;
        loop {
            if self.dbg.step(&self.reg, &self.mem) {
                info!("break");
                break;
            }
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
