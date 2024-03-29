use log::{debug, info};
use mos6502::hexdump;
struct Memory {
    ram: Vec<u8>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory { ram: vec![0; 0] }
    }
}

impl Memory {
    fn load(&self, addr: u16) -> u8 {
        if addr as usize >= self.ram.len() {
            return 0;
        }
        self.ram[addr as usize]
    }
    fn load16(&self, addr: u16) -> u16 {
        let l = self.load(addr) as u16;
        let h = self.load(addr + 1) as u16;
        debug!("mem LOAD16: {:x} {:x}:{:x}", addr, h, l);
        l + (h << 8)
    }
    fn store(&mut self, addr: u16, v: u8) {
        self.ram[addr as usize] = v;
    }
    fn store16(&mut self, addr: u16, v: u16) {
        let l = v as u8;
        let h = (v >> 8) as u8;
        debug!("mem STORE16: {:x} {:x}:{:x}", addr, h, l);
        self.store(addr, l);
        self.store(addr + 1, h);
    }
}

struct Registers {
    pc: u16,
    sp: u16,
    // sr: u8,
    sr: StatusRegister,
    a: u8,
    x: u8,
    y: u8,
}

impl Registers {
    fn adc(&mut self, oper: u8) {
        let res = self.a as u16 + oper as u16 + self.sr.carry();

        self.sr.update_nvzc(res);
        // debug!(
        //     "ADC: {:x} + {:x} = {:x} {}",
        //     self.a, oper, res as u8, self.sr
        // );
        self.a = res as u8;
    }
    fn lda(&mut self, a: u8) {
        self.sr.update_nz(a);
        self.a = a;
    }
    fn and(&mut self, a: u8) {
        self.a = self.a & a;
        self.sr.update_nz(self.a);
    }
}
impl Default for Registers {
    fn default() -> Self {
        Registers {
            pc: 0x600,
            a: 0,
            x: 0,
            y: 0,
            sr: StatusRegister::default(),
            sp: 0xff,
        }
    }
}

impl std::fmt::Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "A=${:02x} X=${:02x} Y={:02x} SP=${:03x} PC=${:03x} SR={}",
            self.a, self.x, self.y, self.sp, self.pc, self.sr
        )
    }
}
struct StatusRegister {
    n: bool,
    v: bool,
    b: bool,
    i: bool,
    d: bool,
    z: bool,
    c: bool,
}
impl Default for StatusRegister {
    fn default() -> Self {
        StatusRegister {
            n: false,
            v: false,
            b: true,
            i: false,
            d: false,
            z: false,
            c: false,
        }
    }
}
impl StatusRegister {
    fn update_nz(&mut self, v: u8) {
        self.n = v >= 0x80;
        self.z = v == 0x0;
    }
    fn update_nvzc(&mut self, v: u16) {
        self.update_nz(v as u8);
        self.c = v > 0xff;
        self.v = self.c;
    }
    fn carry(&self) -> u16 {
        if self.c {
            1
        } else {
            0
        }
    }
}
impl std::fmt::Display for StatusRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn fl(c: char, b: bool) -> char {
            if b {
                c.to_ascii_uppercase()
            } else {
                c
            }
        }
        write!(
            f,
            "{}{}{}{}{}{}{}",
            fl('n', self.n),
            fl('v', self.v),
            fl('b', self.b),
            fl('i', self.i),
            fl('d', self.d),
            fl('z', self.z),
            fl('c', self.c),
        )
    }
}

#[derive(Default)]
struct Cpu {
    reg: Registers,
    mem: Memory,
}

impl Cpu {
    fn load_zeropage(&self) -> u8 {
        let zp_addr = self.mem.load(self.reg.pc + 1);
        let oper = self.mem.load(zp_addr as u16);
        oper
    }
    fn load_absolute(&self) -> u8 {
        let addr = self.mem.load16(self.reg.pc + 1);
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
    fn run(&mut self) {
        // let reg = &mut self.reg;
        // let mem = &mut self.mem;
        loop {
            let opc = self.mem.load(self.reg.pc);
            debug!(
                "pc: {:03x}, opc: {:02x}, reg: {}",
                self.reg.pc, opc, self.reg
            );
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
                0xa5 => {
                    self.reg.lda(self.load_zeropage());
                    2
                }
                0xa9 => {
                    self.reg.lda(self.load_immediate());
                    2
                }
                0xad => {
                    self.reg.lda(self.load_absolute());
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
                0xaa => {
                    debug!("TAX");
                    self.reg.x = self.reg.a;
                    self.reg.sr.update_nvzc(self.reg.x as u16);
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
                    break;
                }
                _ => panic!("unhandled opcode: {:x} pc: {:x}", opc, self.reg.pc),
            };
            self.reg.pc += size as u16;
        }
    }
}

fn main() {
    // env_logger::init();
    simple_logging::log_to(std::io::stdout(), log::LevelFilter::Debug);
    info!("test");
    let mut cpu = Cpu::default();
    cpu.mem.ram = hexdump::read();
    cpu.run();

    // println!("data: {:?}", data);
    hexdump::dump(&cpu.mem.ram);
}
