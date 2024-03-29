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
            "A=${:x} X=${:x} Y={:x} SP=${:x} PC=${:x} SR={}",
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
    fn update_nvzc(&mut self, v: u16) {
        self.n = v as u8 >= 0x80;
        self.z = v as u8 == 0x0;
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
    fn run(&mut self) {
        let reg = &mut self.reg;
        let mem = &mut self.mem;
        loop {
            let opc = mem.load(reg.pc);
            debug!("pc: {:x}, opc: {:x}, reg: {}", reg.pc, opc, reg);
            let size = match opc {
                0x18 => {
                    debug!("CLC");
                    reg.sr.c = false;
                    // reg.sr = reg.sr & !FL_C;
                    1
                }
                0x20 => {
                    let ret = reg.pc + 2;
                    mem.store16(reg.sp as u16 + 0x100, ret);
                    reg.sp -= 2;
                    reg.pc = mem.load16(reg.pc + 1);
                    debug!("JSR -> {:x} {:x}", reg.pc, ret);
                    0
                }
                0x29 => {
                    reg.a = reg.a & mem.load(reg.pc + 1);
                    2
                }
                0x60 => {
                    reg.sp += 2;
                    reg.pc = mem.load16(reg.sp as u16 + 0x100);
                    debug!("RTS -> {:x}", reg.pc);
                    1
                }
                0x65 => {
                    debug!("ADC ZP");
                    let zp_addr = mem.load(reg.pc + 1);
                    let oper = mem.load(zp_addr as u16);
                    let res = reg.a as u16 + oper as u16 + reg.sr.carry();
                    reg.sr.update_nvzc(res);
                    reg.a = res as u8;
                    2
                }
                0x69 => {
                    let oper = mem.load(reg.pc + 1);
                    let res = reg.a as u16 + oper as u16 + reg.sr.carry();

                    reg.sr.update_nvzc(res);
                    debug!("ADC: {:x} + {:x} = {:x} {}", reg.a, oper, res as u8, reg.sr);
                    reg.a = res as u8;
                    2
                }
                0xa5 => {
                    reg.a = mem.load(mem.load(reg.pc + 1) as u16);
                    reg.sr.update_nvzc(reg.a as u16);
                    debug!("LDA ZP {:x} {:x}", mem.load(reg.pc + 1), reg.a);
                    2
                }
                0xa9 => {
                    reg.a = mem.load(reg.pc + 1);
                    reg.sr.update_nvzc(reg.a as u16);
                    debug!("LDA. {}", reg);
                    2
                }
                0x85 => {
                    debug!("STA ZP");
                    mem.store(mem.load(reg.pc + 1) as u16, reg.a);
                    2
                }
                0x8d => {
                    debug!("STA");
                    mem.store(mem.load16(reg.pc + 1), reg.a);
                    3
                }
                0xaa => {
                    debug!("TAX");
                    reg.x = reg.a;
                    reg.sr.update_nvzc(reg.x as u16);
                    1
                }
                0xe8 => {
                    debug!("INX");
                    reg.x = reg.x.wrapping_add(1);
                    1
                }
                0 => {
                    println!("break on 00");
                    break;
                }
                _ => panic!("unhandled opcode: {:x} pc: {:x}", opc, reg.pc),
            };
            reg.pc += size as u16;
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
