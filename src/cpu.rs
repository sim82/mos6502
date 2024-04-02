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
        let (_, size) = match opc {
            0x18 => {
                debug!("CLC");
                self.reg.sr.c = false;
                // reg.sr = reg.sr & !FL_C;
                ((), 1)
            }
            0x20 => {
                let ret = self.reg.pc + 2;
                self.mem.store16(self.reg.sp as u16 + 0x100, ret);
                self.reg.sp -= 2;
                self.reg.pc = self.mem.load16(self.reg.pc + 1);
                debug!("JSR -> {:x} {:x}", self.reg.pc, ret);
                // 0
                ((), 0)
            }
            0x4c => {
                self.reg.pc = self.mem.load16(self.reg.pc + 1);
                ((), 0)
            }
            0x60 => {
                self.reg.sp += 2;
                self.reg.pc = self.mem.load16(self.reg.sp as u16 + 0x100);
                debug!("RTS -> {:x}", self.reg.pc);
                ((), 1)
            }
            0x65 => (self.reg.adc(self.load_zeropage()), 2),
            0x69 => (self.reg.adc(self.load_immediate()), 2),
            0x6d => (self.reg.adc(self.load_absolute()), 3),
            // AND  AND Memory with Accumulator

            //      A AND M -> A                     N Z C I D V
            //                                       + + - - - -

            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      immidiate     AND #oper     29    2     2
            //      zeropage      AND oper      25    2     3
            //      zeropage,X    AND oper,X    35    2     4
            //      absolute      AND oper      2D    3     4
            //      absolute,X    AND oper,X    3D    3     4*
            //      absolute,Y    AND oper,Y    39    3     4*
            //      (indirect,X)  AND (oper,X)  21    2     6
            //      (indirect),Y  AND (oper),Y  31    2     5*
            0x29 => (self.reg.and(self.load_immediate()), 2),
            0x25 => (self.reg.and(self.load_zeropage()), 2),
            0x35 => (self.reg.and(self.load_zeropage_x()), 2),
            0x2d => (self.reg.and(self.load_absolute()), 3),
            0x3d => (self.reg.and(self.load_absolute_x()), 3),
            0x39 => (self.reg.and(self.load_absolute_y()), 3),
            0x21 => (self.reg.and(self.load_indirect_x()), 2),
            0x31 => (self.reg.and(self.load_indirect_y()), 2),
            // ORA  OR Memory with Accumulator
            //      A OR M -> A                      N Z C I D V
            //                                       + + - - - -
            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      immidiate     ORA #oper     09    2     2
            //      zeropage      ORA oper      05    2     3
            //      zeropage,X    ORA oper,X    15    2     4
            //      absolute      ORA oper      0D    3     4
            //      absolute,X    ORA oper,X    1D    3     4*
            //      absolute,Y    ORA oper,Y    19    3     4*
            //      (indirect,X)  ORA (oper,X)  01    2     6
            //      (indirect),Y  ORA (oper),Y  11    2     5*
            0x09 => (self.reg.ora(self.load_immediate()), 2),
            0x05 => (self.reg.ora(self.load_zeropage()), 2),
            0x15 => (self.reg.ora(self.load_zeropage_x()), 2),
            0x0d => (self.reg.ora(self.load_absolute()), 3),
            0x1d => (self.reg.ora(self.load_absolute_x()), 3),
            0x19 => (self.reg.ora(self.load_absolute_y()), 3),
            0x01 => (self.reg.ora(self.load_indirect_x()), 2),
            0x11 => (self.reg.ora(self.load_indirect_y()), 2),
            // EOR  Exclusive-OR Memory with Accumulator
            //      A EOR M -> A                     N Z C I D V
            //                                       + + - - - -
            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      immidiate     EOR #oper     49    2     2
            //      zeropage      EOR oper      45    2     3
            //      zeropage,X    EOR oper,X    55    2     4
            //      absolute      EOR oper      4D    3     4
            //      absolute,X    EOR oper,X    5D    3     4*
            //      absolute,Y    EOR oper,Y    59    3     4*
            //      (indirect,X)  EOR (oper,X)  41    2     6
            //      (indirect),Y  EOR (oper),Y  51    2     5*
            0x49 => (self.reg.eor(self.load_immediate()), 2),
            0x45 => (self.reg.eor(self.load_zeropage()), 2),
            0x55 => (self.reg.eor(self.load_zeropage_x()), 2),
            0x4d => (self.reg.eor(self.load_absolute()), 3),
            0x5d => (self.reg.eor(self.load_absolute_x()), 3),
            0x59 => (self.reg.eor(self.load_absolute_y()), 3),
            0x41 => (self.reg.eor(self.load_indirect_x()), 2),
            0x51 => (self.reg.eor(self.load_indirect_y()), 2),
            // CMP  Compare Memory with Accumulator
            //      A - M                            N Z C I D V
            //                                     + + + - - -
            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      immidiate     CMP #oper     C9    2     2
            //      zeropage      CMP oper      C5    2     3
            //      zeropage,X    CMP oper,X    D5    2     4
            //      absolute      CMP oper      CD    3     4
            //      absolute,X    CMP oper,X    DD    3     4*
            //      absolute,Y    CMP oper,Y    D9    3     4*
            //      (indirect,X)  CMP (oper,X)  C1    2     6
            //      (indirect),Y  CMP (oper),Y  D1    2     5*
            0xcd => {
                //
                let oper = self.load_absolute();
                debug!("cmp: {:x} {:x}", oper, self.reg.a);
                let cmp = self.reg.a.wrapping_sub(oper);

                self.reg.sr.update_nz(cmp);
                ((), 3)
            }
            // BEQ  Branch on Result Zero
            //      branch on Z = 1                  N Z C I D V
            //                                       - - - - - -
            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      relative      BEQ oper      F0    2     2**
            0xf0 => {
                if self.reg.sr.z {
                    let offs = self.mem.load(self.reg.pc + 1) as i8;
                    debug!("BEQ -> {}", offs);
                    self.reg.pc = self.reg.pc.wrapping_add_signed(offs.into());
                } else {
                    debug!("BEQ no");
                }
                ((), 2)
            }
            // STA  Store Accumulator in Memory
            //      A -> M                           N Z C I D V
            //                                       - - - - - -
            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      zeropage      STA oper      85    2     3
            //      zeropage,X    STA oper,X    95    2     4
            //      absolute      STA oper      8D    3     4
            //      absolute,X    STA oper,X    9D    3     5
            //      absolute,Y    STA oper,Y    99    3     5
            //      (indirect,X)  STA (oper,X)  81    2     6
            //      (indirect),Y  STA (oper),Y  91    2     6
            0x85 => (self.store_zeropage(self.reg.a), 2),
            0x95 => (self.store_zeropage_x(self.reg.a), 2),
            0x8d => (self.store_absolute(self.reg.a), 3),
            0x9d => (self.store_absolute_x(self.reg.a), 3),
            0x99 => (self.store_absolute_y(self.reg.a), 3),
            0x91 => (self.store_indirect_y(self.reg.a), 2),
            0x81 => (self.store_indirect_x(self.reg.a), 2),
            // STX  Store Index X in Memory
            //      X -> M                           N Z C I D V
            //                                       - - - - - -
            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      zeropage      STX oper      86    2     3
            //      zeropage,Y    STX oper,Y    96    2     4
            //      absolute      STX oper      8E    3     4
            0x86 => (self.store_zeropage(self.reg.x), 2),
            0x96 => (self.store_zeropage_y(self.reg.x), 2),
            0x8e => (self.store_absolute(self.reg.x), 3),
            // STY  Sore Index Y in Memory
            //      Y -> M                           N Z C I D V
            //                                       - - - - - -
            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      zeropage      STY oper      84    2     3
            //      zeropage,X    STY oper,X    94    2     4
            //      absolute      STY oper      8C    3     4
            0x84 => (self.store_zeropage(self.reg.y), 2),
            0x94 => (self.store_zeropage_x(self.reg.y), 2),
            0x8c => (self.store_absolute(self.reg.y), 3),
            // LDA  Load Accumulator with Memory
            // M -> A                           N Z C I D V
            //                                  + + - - - -
            // addressing    assembler    opc  bytes  cyles
            // --------------------------------------------
            // immidiate     LDA #oper     A9    2     2
            // zeropage      LDA oper      A5    2     3
            // zeropage,X    LDA oper,X    B5    2     4
            // absolute      LDA oper      AD    3     4
            // absolute,X    LDA oper,X    BD    3     4*
            // absolute,Y    LDA oper,Y    B9    3     4*
            // (indirect,X)  LDA (oper,X)  A1    2     6
            // (indirect),Y  LDA (oper),Y  B1    2     5*
            0xa9 => (self.reg.lda(self.load_immediate()), 2),
            0xa5 => (self.reg.lda(self.load_zeropage()), 2),
            0xb5 => (self.reg.lda(self.load_zeropage_x()), 2),
            0xad => (self.reg.lda(self.load_absolute()), 3),
            0xbd => (self.reg.lda(self.load_absolute_x()), 3),
            0xb9 => (self.reg.lda(self.load_absolute_y()), 3),
            0xb1 => (self.reg.lda(self.load_indirect_y()), 2),
            0xa1 => (self.reg.lda(self.load_indirect_x()), 2),

            // LDY  Load Index Y with Memory
            //      M -> Y                           N Z C I D V
            //                                       + + - - - -
            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      immidiate     LDY #oper     A0    2     2
            //      zeropage      LDY oper      A4    2     3
            //      zeropage,X    LDY oper,X    B4    2     4
            //      absolute      LDY oper      AC    3     4
            //      absolute,X    LDY oper,X    BC    3     4*
            0xa0 => (self.reg.ldy(self.load_immediate()), 2),
            0xa4 => (self.reg.ldy(self.load_zeropage()), 2),
            0xb4 => (self.reg.ldy(self.load_zeropage_x()), 2),
            0xac => (self.reg.ldy(self.load_absolute()), 3),
            0xbc => (self.reg.ldy(self.load_absolute_x()), 3),
            // LDX  Load Index X with Memory
            //      M -> X                           N Z C I D V
            //                                       + + - - - -
            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      immidiate     LDX #oper     A2    2     2
            //      zeropage      LDX oper      A6    2     3
            //      zeropage,Y    LDX oper,Y    B6    2     4
            //      absolute      LDX oper      AE    3     4
            //      absolute,Y    LDX oper,Y    BE    3     4*
            0xa2 => (self.reg.ldx(self.load_immediate()), 2),
            0xa6 => (self.reg.ldx(self.load_zeropage()), 2),
            0xb6 => (self.reg.ldx(self.load_zeropage_y()), 2),
            0xae => (self.reg.ldx(self.load_absolute()), 3),
            0xbe => (self.reg.ldx(self.load_absolute_y()), 3),

            // INC  Increment Memory by One
            //      M + 1 -> M                       N Z C I D V
            //                                       + + - - - -
            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      zeropage      INC oper      E6    2     5
            //      zeropage,X    INC oper,X    F6    2     6
            //      absolute      INC oper      EE    3     6
            //      absolute,X    INC oper,X    FE    3     7
            0xe6 => {
                let v = self.load_zeropage().wrapping_add(1);
                self.reg.sr.update_nz(v);
                self.store_zeropage(v);
                ((), 2)
            }
            0xf6 => {
                let v = self.load_zeropage_x().wrapping_add(1);
                self.reg.sr.update_nz(v);
                self.store_zeropage_x(v);
                ((), 2)
            }

            0xee => {
                let v = self.load_absolute().wrapping_add(1);
                self.reg.sr.update_nz(v);
                self.store_absolute(v);
                ((), 3)
            }
            0xfe => {
                let v = self.load_absolute_x().wrapping_add(1);
                self.reg.sr.update_nz(v);
                self.store_absolute_x(v);
                ((), 3)
            }
            // DEC  Decrement Memory by One
            // M - 1 -> M                       N Z C I D V
            //                                  + + - - - -
            // addressing    assembler    opc  bytes  cyles
            // --------------------------------------------
            // zeropage      DEC oper      C6    2     5
            // zeropage,X    DEC oper,X    D6    2     6
            // absolute      DEC oper      CE    3     6
            // absolute,X    DEC oper,X    DE    3     7
            0xc6 => {
                let v = self.load_zeropage().wrapping_sub(1);
                self.reg.sr.update_nz(v);
                self.store_zeropage(v);
                ((), 2)
            }
            0xd6 => {
                let v = self.load_zeropage_x().wrapping_sub(1);
                self.reg.sr.update_nz(v);
                self.store_zeropage_x(v);
                ((), 2)
            }

            0xce => {
                let v = self.load_absolute().wrapping_sub(1);
                self.reg.sr.update_nz(v);
                self.store_absolute(v);
                ((), 3)
            }
            0xde => {
                let v = self.load_absolute_x().wrapping_sub(1);
                self.reg.sr.update_nz(v);
                self.store_absolute_x(v);
                ((), 3)
            }
            // LSR  Shift One Bit Right (Memory or Accumulator)

            //      0 -> [76543210] -> C             N Z C I D V
            //                                       0 + + - - -

            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      accumulator   LSR A         4A    1     2
            //      zeropage      LSR oper      46    2     5
            //      zeropage,X    LSR oper,X    56    2     6
            //      absolute      LSR oper      4E    3     6
            //      absolute,X    LSR oper,X    5E    3     7
            0x4a => {
                self.reg.a = self.reg.lsr(self.reg.a);
                ((), 1)
            }

            0x46 => {
                let addr = self.addr_zeropage() as u16;
                let v = self.reg.lsr(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 2)
            }
            0x56 => {
                let addr = self.addr_zeropage_x() as u16;
                let v = self.reg.lsr(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 2)
            }
            0x4e => {
                let addr = self.addr_absolute();
                let v = self.reg.lsr(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 3)
            }
            0x5e => {
                let addr = self.addr_absolute_x();
                let v = self.reg.lsr(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 3)
            }

            // ASL  Shift Left One Bit (Memory or Accumulator)

            //      C <- [76543210] <- 0             N Z C I D V
            //                                       + + + - - -

            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      accumulator   ASL A         0A    1     2
            //      zeropage      ASL oper      06    2     5
            //      zeropage,X    ASL oper,X    16    2     6
            //      absolute      ASL oper      0E    3     6
            //      absolute,X    ASL oper,X    1E    3     7
            0x0a => {
                self.reg.a = self.reg.asl(self.reg.a);
                ((), 1)
            }

            0x06 => {
                let addr = self.addr_zeropage() as u16;
                let v = self.reg.asl(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 2)
            }
            0x16 => {
                let addr = self.addr_zeropage_x() as u16;
                let v = self.reg.asl(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 2)
            }
            0x0e => {
                let addr = self.addr_absolute();
                let v = self.reg.asl(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 3)
            }
            0x1e => {
                let addr = self.addr_absolute_x();
                let v = self.reg.asl(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 3)
            }
            // ROL  Rotate One Bit Left (Memory or Accumulator)
            //      C <- [76543210] <- C             N Z C I D V
            //                                       + + + - - -
            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      accumulator   ROL A         2A    1     2
            //      zeropage      ROL oper      26    2     5
            //      zeropage,X    ROL oper,X    36    2     6
            //      absolute      ROL oper      2E    3     6
            //      absolute,X    ROL oper,X    3E    3     7
            0x2a => {
                self.reg.a = self.reg.rol(self.reg.a);
                ((), 1)
            }

            0x26 => {
                let addr = self.addr_zeropage() as u16;
                let v = self.reg.rol(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 2)
            }
            0x36 => {
                let addr = self.addr_zeropage_x() as u16;
                let v = self.reg.rol(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 2)
            }
            0x2e => {
                let addr = self.addr_absolute();
                let v = self.reg.rol(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 3)
            }
            0x3e => {
                let addr = self.addr_absolute_x();
                let v = self.reg.rol(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 3)
            }
            // ROR  Rotate One Bit Right (Memory or Accumulator)
            //      C -> [76543210] -> C             N Z C I D V
            //                                       + + + - - -
            //      addressing    assembler    opc  bytes  cyles
            //      --------------------------------------------
            //      accumulator   ROR A         6A    1     2
            //      zeropage      ROR oper      66    2     5
            //      zeropage,X    ROR oper,X    76    2     6
            //      absolute      ROR oper      6E    3     6
            //      absolute,X    ROR oper,X    7E    3     7
            0x6a => {
                self.reg.a = self.reg.ror(self.reg.a);
                ((), 1)
            }

            0x66 => {
                let addr = self.addr_zeropage() as u16;
                let v = self.reg.ror(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 2)
            }
            0x76 => {
                let addr = self.addr_zeropage_x() as u16;
                let v = self.reg.ror(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 2)
            }
            0x6e => {
                let addr = self.addr_absolute();
                let v = self.reg.ror(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 3)
            }
            0x7e => {
                let addr = self.addr_absolute_x();
                let v = self.reg.ror(self.mem.load(addr));
                self.mem.store(addr, v);
                ((), 3)
            }
            0xaa => {
                debug!("TAX");
                self.reg.x = self.reg.a;
                self.reg.sr.update_nz(self.reg.x);
                ((), 1)
            }
            0xe8 => {
                debug!("INX");
                let res = (self.reg.x as u16).wrapping_add(1);
                self.reg.sr.update_nvzc(res);
                self.reg.x = res as u8;

                ((), 1)
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
        let zp_addr = self.addr_zeropage();
        let oper = self.mem.load(zp_addr as u16);
        oper
    }

    fn load_zeropage_x(&self) -> u8 {
        let addr = self.addr_zeropage_x();
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
        let addr = self.addr_absolute();
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

    fn addr_zeropage(&self) -> u8 {
        let zp_addr = self.mem.load(self.reg.pc + 1);
        zp_addr
    }
    fn addr_zeropage_x(&self) -> u8 {
        let zp_addr = self.mem.load(self.reg.pc + 1);
        let addr = zp_addr.wrapping_add(self.reg.x);
        addr
    }
    fn addr_absolute(&self) -> u16 {
        let addr = self.mem.load16(self.reg.pc + 1);
        addr
    }
    fn addr_absolute_x(&self) -> u16 {
        let addr = self.mem.load16(self.reg.pc + 1);
        let addr = addr
            .wrapping_add(self.reg.x as u16)
            // .wrapping_add(self.reg.sr.carry())
        ;
        addr
    }
    fn addr_absolute_y(&self) -> u16 {
        let addr = self.mem.load16(self.reg.pc + 1);
        let addr = addr
            .wrapping_add(self.reg.y as u16)
            // .wrapping_add(self.reg.sr.carry())
        ;
        addr
    }
    fn addr_indirect_y(&self) -> u16 {
        let zp_addr = self.mem.load(self.reg.pc + 1);
        let addr = self.mem.load16(zp_addr as u16);

        debug!("zp_addr: {:x} {:x}", zp_addr, addr);
        let eff_addr = addr
            .wrapping_add(self.reg.y as u16)
            // .wrapping_add(self.reg.sr.carry())
        ;
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
