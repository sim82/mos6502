use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
    io::{Bytes, Read, Stdout, Write},
};

use rand::Rng;
use termion::{
    async_stdin,
    raw::{IntoRawMode, RawTerminal},
    AsyncReader,
};

use crate::{
    mem::{self, Memory},
    reg::Registers,
};

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
        mem.store(0xfe, rand::thread_rng().gen());
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
pub struct Apple1Pia {
    stdout: RawTerminal<Stdout>,
    input: Bytes<AsyncReader>,
    trigger_pc: u16,
    textbuf: [[u8; 80]; 10],
    outline: usize,
    outcol: usize,
    screen_dirty: bool,
    monitor_lastchunks: HashMap<u16, u64>,
    keyb_strobe: usize,
}
impl Default for Apple1Pia {
    fn default() -> Self {
        Self {
            stdout: std::io::stdout().into_raw_mode().unwrap(),
            input: async_stdin().bytes(),
            trigger_pc: 0x734,
            textbuf: [[0x20u8; 80]; 10],
            outcol: 0,
            outline: 0,
            screen_dirty: true,
            monitor_lastchunks: Default::default(),
            keyb_strobe: 0,
        }
    }
}
impl Dbg for Apple1Pia {
    fn step(&mut self, reg: &mut Registers, mem: &mut Memory) -> bool {
        if self.keyb_strobe != 0 {
            self.keyb_strobe -= 1;
            if self.keyb_strobe == 0 {
                mem.store(0xd011, mem.load(0xd011) & 0b1111111);
            }
        }
        loop {
            let Some(key) = self.input.next() else { break };
            match key.unwrap() {
                0x1b => return true,
                c if c.is_ascii() => {
                    // let c = if c == 0xd { 0xa } else { c };
                    let v = mem.load(0xd11);
                    // println!("key: {:x} {:x}", c, v);
                    mem.store(0xd011, v | 0b10000000);
                    mem.store(0xd010, (c as u8) | 0b10000000);
                    self.keyb_strobe = 2;
                }

                _ => (),
            }
        }
        // check display register
        let v = mem.load(0xd012);
        // if (v & 0b10000000) != 0 {
        if v != 0x0 {
            let c = v & 0b1111111;
            // print!("{}", c as char);
            // self.stdout.flush().unwrap();
            self.putc(c);
            mem.store(0xd012, 0x0);
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
        self.draw_textbuf(1, 1);
        self.draw_monitor(10, mem);
        self.stdout.flush().unwrap();
        false
    }
}
impl Apple1Pia {
    pub fn putc(&mut self, c: u8) {
        if c == 0x0d {
            // self.outline += 1;
            self.outcol = 0;
        }
        if c == 0x0a {
            self.outline += 1;
        }
        if self.outcol >= 80 {
            self.outcol = 0;
            self.outline += 1;
        }
        if self.outline >= 10 {
            self.outline = 9;
            self.textbuf.copy_within(1..9, 0);
            self.textbuf[9] = [0x20u8; 80];
        }
        self.textbuf[self.outline][self.outcol] = c;
        self.outcol += 1;
        self.screen_dirty = true;
    }
    pub fn draw_textbuf(&mut self, screenline: u16, screencol: u16) {
        if !self.screen_dirty {
            return;
        }
        for (y, line) in self.textbuf.iter().enumerate() {
            write!(
                self.stdout,
                "{}{}",
                termion::cursor::Goto(screencol, (y as u16) + screenline),
                termion::clear::CurrentLine
            )
            .unwrap();
            for (x, c) in line.iter().enumerate() {
                write!(self.stdout, "{}", *c as char).unwrap();
            }
        }
        self.screen_dirty = false;
    }
    pub fn draw_monitor(&mut self, mut screenline: u16, mem: &mem::Memory) {
        //
        let Ok((_width, height)) = termion::terminal_size() else {
            return;
        };
        let base_address = 0x00;
        let chunk_size = 16;
        for (i, chunk) in mem.get()[base_address..].chunks(chunk_size).enumerate() {
            if screenline >= height {
                break;
            }
            if chunk.iter().any(|c| *c != 0) {
                let mut hasher = DefaultHasher::default();
                chunk.hash(&mut hasher);
                let chunk_hash = hasher.finish();
                if self.monitor_lastchunks.get(&screenline) != Some(&chunk_hash) {
                    write!(
                        self.stdout,
                        "{}{}",
                        termion::cursor::Goto(1, screenline),
                        termion::clear::CurrentLine
                    )
                    .unwrap();
                    write!(
                        self.stdout,
                        "{:04x}: {}",
                        i * chunk_size,
                        chunk
                            .iter()
                            .map(|b| format!("{:02x}", *b))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                    .unwrap();
                    self.monitor_lastchunks.insert(screenline, chunk_hash);
                }
                screenline += 1;
            }
        }
    }
}
