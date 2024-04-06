use std::io::{Bytes, Read, Stdout, Write};

use termion::{
    async_stdin,
    raw::{IntoRawMode, RawTerminal},
    AsyncReader,
};

use crate::{mem::Memory, reg::Registers};

pub trait Dbg {
    fn step(&mut self, reg: &mut Registers, mem: &mut Memory) -> bool;
}
pub struct CycleDetect {
    pc_trace: [u8; 0xffff],
}

impl Default for CycleDetect {
    fn default() -> Self {
        Self {
            pc_trace: [0u8; 0xffff],
        }
    }
}

impl Dbg for CycleDetect {
    fn step(&mut self, reg: &mut Registers, _mem: &mut Memory) -> bool {
        self.pc_trace[reg.pc as usize] += 1;
        if self.pc_trace[reg.pc as usize] > 2 {
            println!("cycle");
            true
        } else {
            false
        }
    }
}
pub struct DumpScreen {
    stdout: RawTerminal<Stdout>,
    input: Bytes<AsyncReader>,
    trigger_pc: u16,
}
impl Drop for DumpScreen {
    fn drop(&mut self) {
        println!("drop");
    }
}
impl Default for DumpScreen {
    fn default() -> Self {
        Self {
            stdout: std::io::stdout().into_raw_mode().unwrap(),
            input: async_stdin().bytes(),
            trigger_pc: 0x734,
        }
    }
}
impl Dbg for DumpScreen {
    fn step(&mut self, reg: &mut Registers, mem: &mut Memory) -> bool {
        loop {
            let Some(key) = self.input.next() else { break };

            println!("key: {:?}", key);
            match key.unwrap() {
                c if c.is_ascii() => mem.store(0xff, c as u8),
                _ => (),
            }
        }
        if self.trigger_pc != 0 && reg.pc == self.trigger_pc {
            let mut offs = 0x200u16;
            write!(
                self.stdout,
                "{}{}",
                termion::clear::All,
                termion::cursor::Hide
            )
            .unwrap();
            for y in 0..32 {
                write!(self.stdout, "{}", termion::cursor::Goto(1, y + 1)).unwrap();
                for _x in 0..32 {
                    let pixel = mem.load(offs);
                    offs += 1;
                    print!("{}", if pixel == 0 { ' ' } else { 'X' });
                }
                println!();
            }
            println!("PC={:x}", reg.pc);
            writeln!(self.stdout, "{}", termion::cursor::Show).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        false
    }
}
