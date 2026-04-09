
pub struct Zo(u64);

impl From<(u32, u32)> for Zo {
    fn from(xy: (u32, u32)) -> Self {
        Self(obtain_z_order(xy))
    }
}

impl Zo {
    pub fn in_range(&self, d: u32) -> bool {
        //lets say D is 3, then the biggest allowed morton code is
        //`111111`, i.e. D*2 bits set to 1.
        //thus, we just have to check that the number is less than 2 ^ (D*2).
        self.0 < 2u64.pow(d * 2)
    }
    
    pub fn get_cell(&self, depth: u32) -> usize {
        let v = self.0 as usize;
        let d = ((depth - 1) * 2) as usize;
        (v >> d) & 0b11
    }
    
}

pub fn get_cell_xy(x: u32, y: u32, depth: u32) -> usize {
    let d = depth - 1;
    let dx = (x >> d) & 1;
    let dy = (y >> d) & 1;
    let i = (dx + dy * 2) as usize;
    i
}

#[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
fn obtain_z_order((x, y): (u32, u32)) -> u64 {
    use std::arch::x86_64::_pdep_u64;
    _pdep_u64(x, 0x5555_5555_5555_5555)+ _pdep_u64(y, 0xaaaa_aaaa_aaaa_aaaa)
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "bmi2")))]
fn obtain_z_order((x, y): (u32, u32)) -> u64 {
    interleave_u32(x) | (interleave_u32(y) << 1)
}

pub const fn interleave_u32(v: u32) -> u64 {
    let v = v as u64;
    let v = (v ^ (v << 16)) & 0x0000_ffff_0000_ffff;
    let v = (v ^ (v <<  8)) & 0x00ff_00ff_00ff_00ff;
    let v = (v ^ (v <<  4)) & 0x0f0f_0f0f_0f0f_0f0f;
    let v = (v ^ (v <<  2)) & 0x3333_3333_3333_3333;
    let v = (v ^ (v <<  1)) & 0x5555_5555_5555_5555;
    v
}