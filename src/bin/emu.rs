use log::info;
use mos6502::hexdump;
use mos6502::{cpu::Cpu, mem::Memory};

fn main() {
    // env_logger::init();
    simple_logging::log_to(std::io::stdout(), log::LevelFilter::Debug);
    info!("test");
    // let mem = Memory::new(hexdump::read());
    // let mem = Memory::new(hexdump::read_bin(
    //     "/home/sim/src/FPGA-netlist-tools/6502-test-code/test1.bin",
    //     0xfff0,
    // ));

    let mem = Memory::new(hexdump::read_bin("6502-test-code/AllSuiteA.bin", 0x4000));
    let mut cpu = Cpu::new(mem);
    // cpu.set_pc(0xfff0);
    cpu.set_pc(0x4000);

    cpu.dump_mem();
    cpu.run();

    // println!("data: {:?}", data);
    cpu.dump_mem();
}
