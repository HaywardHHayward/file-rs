use crate::utf::*;
pub struct Utf8Sequence {
    full_length: u8,
    current_length: u8,
    bytes: [u8; 4],
}

impl Utf for Utf8Sequence {
    type Point = u8;
    type Codepoint = u32;

    fn get_codepoint(&self) -> Self::Codepoint {
        let mut codepoint: u32 = self.bytes[0] as u32;
        match self.full_length {
            1 => {}
            2 => codepoint ^= 0b1100_0000,
            3 => codepoint ^= 0b1110_0000,
            4 => codepoint ^= 0b1111_0000,
            _ => unreachable!(),
        }
        for i in 1..self.full_length {
            codepoint = (codepoint << 6) | ((self.bytes[i as usize] ^ 0b10_000000) as u32);
        }
        codepoint
    }
    fn add_point(&mut self, point: Self::Point) -> bool {
        if self.current_length + 1 > self.full_length {
            return false;
        }
        if point >= 0b1100_0000 || Self::is_invalid(point) {
            return false;
        }
        self.bytes[self.current_length as usize] = point;
        self.current_length += 1;
        true
    }
    fn is_valid(&self) -> bool {
        let codepoint = self.get_codepoint();
        if !is_valid_codepoint(codepoint) {
            return false;
        }
        match self.full_length {
            1 => (..=0x7F).contains(&codepoint),
            2 => (0x80..=0x7FF).contains(&codepoint),
            3 => (0x800..=0xFFFF).contains(&codepoint),
            4 => (0x100000..=0x10FFFF).contains(&codepoint),
            _ => unreachable!(),
        }
    }
}

impl Utf8Sequence {
    pub const fn build(byte: u8) -> Option<Self> {
        let full_length = match byte.leading_ones() {
            0 => 1,
            n @ 2..=4 => n,
            _ => return None,
        } as u8;
        if Self::is_invalid(byte) {
            return None;
        }
        let mut bytes: [u8; 4] = [0; 4];
        bytes[0] = byte;
        Some(Self {
            full_length,
            current_length: 1,
            bytes,
        })
    }
    const fn is_invalid(byte: u8) -> bool {
        matches!(byte, 0xC0 | 0xC1 | 0xF5..)
    }
    pub const fn current_len(&self) -> usize {
        self.current_length as usize
    }
    pub const fn full_len(&self) -> usize {
        self.full_length as usize
    }
}
