pub struct Registers {
    pub pc: u16,
    pub sp: u16,
    pub sr: StatusRegister,
    pub a: u8,
    pub x: u8,
    pub y: u8,
}

impl Registers {
    pub fn adc(&mut self, oper: u8) {
        let res = self.a as u16 + oper as u16 + self.sr.carry();

        self.sr.update_nvzc(res);
        // debug!(
        //     "ADC: {:x} + {:x} = {:x} {}",
        //     self.a, oper, res as u8, self.sr
        // );
        self.a = res as u8;
    }
    pub fn lda(&mut self, a: u8) {
        self.sr.update_nz(a);
        self.a = a;
    }
    pub fn and(&mut self, a: u8) {
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
pub struct StatusRegister {
    pub n: bool,
    pub v: bool,
    pub b: bool,
    pub i: bool,
    pub d: bool,
    pub z: bool,
    pub c: bool,
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
    pub fn update_nz(&mut self, v: u8) {
        self.n = v >= 0x80;
        self.z = v == 0x0;
    }
    pub fn update_nvzc(&mut self, v: u16) {
        self.update_nz(v as u8);
        self.c = v > 0xff;
        self.v = self.c;
    }
    pub fn carry(&self) -> u16 {
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
