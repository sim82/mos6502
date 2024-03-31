use log::debug;
pub struct Memory {
    ram: Vec<u8>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory { ram: vec![0; 0] }
    }
}

impl Memory {
    pub fn new(ram: Vec<u8>) -> Self {
        Self { ram }
    }
    pub fn get(&self) -> &[u8] {
        &self.ram
    }
    pub fn load(&self, addr: u16) -> u8 {
        if addr as usize >= self.ram.len() {
            // debug!("LOAD (uninit): {:x} {:x}", addr, self.ram[addr as usize]);
            return 0;
        }
        // debug!("LOAD: {:x} {:x}", addr, self.ram[addr as usize]);
        self.ram[addr as usize]
    }
    pub fn load16(&self, addr: u16) -> u16 {
        let l = self.load(addr) as u16;
        let h = self.load(addr + 1) as u16;
        // debug!("mem LOAD16: {:x} {:x}:{:x}", addr, h, l);
        l + (h << 8)
    }
    pub fn store(&mut self, addr: u16, v: u8) {
        debug!("mem STORE: {:x} {:x}", addr, v);
        self.ram[addr as usize] = v;
    }
    pub fn store16(&mut self, addr: u16, v: u16) {
        let l = v as u8;
        let h = (v >> 8) as u8;
        debug!("mem STORE16: {:x} {:x}:{:x}", addr, h, l);
        self.store(addr, l);
        self.store(addr + 1, h);
    }
}
