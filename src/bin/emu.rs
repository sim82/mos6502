use log::info;
use mos6502::hexdump;
use mos6502::{cpu::Cpu, mem::Memory};

fn main() {
    // env_logger::init();
    simple_logging::log_to(std::io::stdout(), log::LevelFilter::Debug);
    info!("test");
    let mut cpu = Cpu::new(Memory::new(hexdump::read()));
    cpu.run();

    // println!("data: {:?}", data);
    hexdump::dump(&cpu.get_mem().get());
}
