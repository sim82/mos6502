use log::debug;
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
        debug!("LOAD16: {:x} {:x}:{:x}", addr, h, l);
        l + (h << 8)
    }
    fn store(&mut self, addr: u16, v: u8) {
        self.ram[addr as usize] = v;
    }
    fn store16(&mut self, addr: u16, v: u16) {
        let l = v as u8;
        let h = (v >> 8) as u8;
        debug!("STORE16: {:x} {:x}:{:x}", addr, h, l);
        self.store(addr, l);
        self.store(addr + 1, h);
    }
}

const FL_C: u8 = 0b00100;

struct Registers {
    pc: u16,
    ac: u8,
    x: u8,
    y: u8,
    sr: u8,
    sp: u8,
}
impl Default for Registers {
    fn default() -> Self {
        Registers {
            pc: 0x600,
            ac: 0,
            x: 0,
            y: 0,
            sr: 0,
            sp: 0xff,
        }
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
            let size = match opc {
                0x18 => {
                    debug!("CLC");
                    reg.sr = reg.sr & !FL_C;
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
                    reg.ac = reg.ac & mem.load(reg.pc + 1);
                    2
                }
                0x60 => {
                    reg.sp += 2;
                    reg.pc = mem.load16(reg.sp as u16 + 0x100);
                    debug!("RTS -> {:x}", reg.pc);
                    1
                }
                0x69 => {
                    let oper = mem.load(reg.pc + 1);
                    let res = reg.ac as u16 + oper as u16;

                    if res > 0xff {
                        reg.sr |= FL_C;
                    }
                    debug!(
                        "ADC: {:x} + {:x} = {:x} {:b}",
                        reg.ac, oper, res as u8, reg.sr
                    );
                    reg.ac = res as u8;
                    2
                }
                0xa5 => {
                    reg.ac = mem.load(mem.load(reg.pc + 1) as u16);
                    debug!("LDA ZP {:x} {:x}", mem.load(reg.pc + 1), reg.ac);
                    2
                }
                0xa9 => {
                    debug!("LDA");
                    reg.ac = mem.load(reg.pc + 1);
                    2
                }
                0x85 => {
                    debug!("STA ZP");
                    mem.store(mem.load(reg.pc + 1) as u16, reg.ac);
                    2
                }
                0x8d => {
                    debug!("STA");
                    mem.store(mem.load16(reg.pc + 1), reg.ac);
                    3
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
    env_logger::init();
    let mut cpu = Cpu::default();
    cpu.mem.ram = hexdump::read();
    cpu.run();

    // println!("data: {:?}", data);
    hexdump::dump(&cpu.mem.ram);
}
