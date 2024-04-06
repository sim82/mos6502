pub struct Registers {
    pub pc: u16,
    pub sp: u16,
    pub sr: StatusRegister,
    pub a: u8,
    pub x: u8,
    pub y: u8,
}

fn sign(v: u8) -> bool {
    v > 0x7f
}
impl Registers {
    pub fn adc(&mut self, oper: u8) {
        let res = self.a as u16 + oper as u16 + self.sr.carry();

        // println!("adc: {:x} {:x} {:x}", self.a, oper, self.sr.carry());
        self.sr.update_nz(res as u8);
        self.sr.c = res > 0xff;
        // self.sr.v = sign(self.a) != sign(oper) && sign(self.a) != sign(res as u8);
        // self.sr.v = sign(self.a) != sign(res as u8);

        self.sr.v = (res > 0x7f) != ((res as u8) > 0x7f);
        self.a = res as u8;
    }
    pub fn sbc(&mut self, oper: u8) {
        let res = (self.a as u16)
            .wrapping_sub(oper as u16)
            .wrapping_sub(self.sr.inv_carry());

        self.sr.update_nz(res as u8);
        self.sr.c = res < 0xff;
        // self.sr.v = sign(self.a) != sign(oper) && sign(self.a) != sign(res as u8);
        self.sr.v = (res > 0x7f) != ((res as u8) > 0x7f);
        // self.sr.v = sign(self.a) != sign(res as u8);

        self.a = res as u8;
    }
    pub fn cmp(&mut self, a: u8, b: u8) {
        let res = (a as u16).wrapping_sub(b as u16);
        self.sr.update_nz(res as u8);
        self.sr.c = res > 0xff;
    }
    pub fn ora(&mut self, oper: u8) {
        self.a |= oper;
        self.sr.update_nz(self.a);
    }
    pub fn eor(&mut self, oper: u8) {
        self.a ^= oper;
        self.sr.update_nz(self.a);
    }
    pub fn lda(&mut self, a: u8) {
        self.sr.update_nz(a);
        self.a = a;
    }
    pub fn ldx(&mut self, x: u8) {
        self.sr.update_nz(x);
        self.x = x;
    }
    pub fn ldy(&mut self, y: u8) {
        self.sr.update_nz(y);
        self.y = y;
    }
    pub fn and(&mut self, a: u8) {
        self.a = self.a & a;
        self.sr.update_nz(self.a);
    }
    pub fn lsr(&mut self, v: u8) -> u8 {
        self.sr.c = (v & 0x1) == 0x1;
        let res = v >> 1;
        self.sr.update_nz(res);
        res
    }
    pub fn asl(&mut self, v: u8) -> u8 {
        self.sr.c = (v & 0x80) == 0x80;
        let res = v << 1;
        self.sr.update_nz(res);
        res
    }
    pub fn ror(&mut self, v: u8) -> u8 {
        let oldc = if self.sr.c { 0x80 } else { 0 };
        self.sr.c = (v & 0x1) == 0x1;
        let res = (v >> 1) | oldc;
        self.sr.update_nz(res);
        res
    }
    pub fn rol(&mut self, v: u8) -> u8 {
        let oldc = if self.sr.c { 0x1 } else { 0 };
        self.sr.c = (v & 0x80) == 0x80;
        let res = (v << 1) | oldc;
        self.sr.update_nz(res);
        res
    }
    pub fn bit(&mut self, v: u8) {
        self.sr.n = (v & 0b1000000) != 0;
        self.sr.v = (v & 0b100000) != 0;
        self.sr.z = (self.a & v) == 0;
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
            1 // << 8
        } else {
            0
        }
    }
    pub fn inv_carry(&self) -> u16 {
        if !self.c {
            1 // << 8
        } else {
            0
        }
    }

    pub fn to_u8(&self) -> u8 {
        let mut ret = 0b100000u8; // compatibility with easy 6502 emulator
        if self.c {
            ret |= 0b1;
        }
        if self.z {
            ret |= 0b10;
        }
        if self.i {
            ret |= 0b100;
        }
        if self.d {
            ret |= 0b1000;
        }
        if self.b {
            ret |= 0b10000;
        }
        if self.v {
            ret |= 0b1000000;
        }
        if self.n {
            ret |= 0b10000000;
        }
        ret
    }
    pub fn set_from_u8(&mut self, v: u8) {
        self.c = (v & 0b1) != 0;
        self.z = (v & 0b10) != 0;
        self.i = (v & 0b100) != 0;
        self.d = (v & 0b1000) != 0;
        self.v = (v & 0b1000000) != 0;
        self.n = (v & 0b10000000) != 0;
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
