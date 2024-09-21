use std::vec::Vec;

pub struct Utf8Sequence {
    length: u8,
    bytes: Vec<u8>,
}

impl Utf8Sequence {
    pub fn build(byte: u8) -> Option<Self> {
        let length = match byte.leading_ones() {
            0 => 1,
            n @ 2..=4 => n,
            _ => return None,
        } as u8;
        if Self::is_invalid(byte) {
            return None;
        }
        let mut vec: Vec<u8> = Vec::with_capacity(length as usize);
        vec.push(byte);
        Some(Self { length, bytes: vec })
    }
    pub fn add_byte(&mut self, byte: u8) -> bool {
        if self.bytes.len() + 1 > self.length as usize {
            return false;
        }
        if byte >= 0b1100_0000 || Self::is_invalid(byte) {
            return false;
        }
        self.bytes.push(byte);
        true
    }
    pub const fn is_invalid(byte: u8) -> bool {
        byte == 0xC0 || byte == 0xC1 || byte >= 0xF5
    }
    pub fn get_codepoint(&self) -> Option<u32> {
        let mut codepoint: u32 = self.bytes[0] as u32;
        match self.length {
            1 => {}
            2 => codepoint ^= 0b1100_0000,
            3 => codepoint ^= 0b1110_0000,
            4 => codepoint ^= 0b1111_0000,
            _ => return None,
        }
        if self.bytes.len() != self.length as usize {
            return None;
        }
        for &byte in self.bytes.iter().skip(1) {
            codepoint = (codepoint << 6) | ((byte ^ 0b10_000000) as u32)
        }
        Some(codepoint)
    }
    pub fn is_valid_codepoint(&self) -> bool {
        let Some(codepoint) = self.get_codepoint() else {
            return false;
        };
        match self.length {
            1 => codepoint <= 0x7F,
            2 => codepoint > 0x7F && codepoint <= 0x7FF,
            3 => codepoint > 0x7FF && codepoint <= 0xFFFF,
            4 => codepoint > 0xFFFF && codepoint <= 0x10FFFF,
            _ => unreachable!(),
        }
    }
    
    pub fn current_len(&self) -> usize {
        self.bytes.len()
    }
    pub fn full_len(&self) -> usize {
        self.length as usize
    }
}
