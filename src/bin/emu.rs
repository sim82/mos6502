use log::info;
use mos6502::dbg::{Apple1Pia, CycleDetect, Dbg, DumpScreen};
use mos6502::hexdump;
use mos6502::reg::Registers;
use mos6502::{cpu::Cpu, mem::Memory};

struct DbgNop;

impl Dbg for DbgNop {
    fn step(&mut self, _reg: &mut Registers, _mem: &mut Memory) -> bool {
        false
    }
}
fn main() {
    // env_logger::init();
    simple_logging::log_to(std::io::stdout(), log::LevelFilter::Info);
    info!("test");
    // let mem = Memory::new(hexdump::read());
    // let mem = Memory::new(hexdump::read_bin(
    //     "/home/sim/src/FPGA-netlist-tools/6502-test-code/test1.bin",
    //     0xfff0,
    // ));

    // let start = 0x4000u16;
    // let ram = hexdump::read_bin("6502-test-code/AllSuiteA.bin", start.into());
    // let start = 0x600u16;
    // let ram = hexdump::read_txt("asm/snake.txt");
    let start = 0xe000u16;
    let ram = hexdump::read_bin("6502-test-code/apple1basic.bin", start.into());
    // let mem = Memory::new(hexdump::read_bin("6502-test-code/AllSuiteA.bin", 0x4000));
    // let start = 0x600;

    let mem = Memory::new(ram);
    let mut cpu = Cpu::new(mem);
    // cpu.set_pc(0xfff0);
    cpu.set_pc(start);

    cpu.dump_mem();
    {
        let mut dbg: Box<dyn Dbg> = if false {
            Box::new(DumpScreen::default())
        } else if false {
            Box::new(CycleDetect::default())
        } else if true {
            Box::new(Apple1Pia::default())
        } else {
            Box::new(DbgNop)
        };
        cpu.run(&mut *dbg);
    }
    // println!("data: {:?}", data);
    cpu.dump_mem();
}
