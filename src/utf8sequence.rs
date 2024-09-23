pub struct Utf8Sequence {
    full_length: u8,
    current_length: u8,
    bytes: [u8; 4],
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
    pub fn add_byte(&mut self, byte: u8) -> bool {
        if self.current_length + 1 > self.full_length {
            return false;
        }
        if byte >= 0b1100_0000 || Self::is_invalid(byte) {
            return false;
        }
        self.bytes[self.current_length as usize] = byte;
        self.current_length += 1;
        true
    }
    pub const fn is_invalid(byte: u8) -> bool {
        matches!(byte, 0xC0 | 0xC1 | 0xF5..)
    }
    pub fn get_codepoint(&self) -> Option<u32> {
        let mut codepoint: u32 = self.bytes[0] as u32;
        match self.full_length {
            1 => {}
            2 => codepoint ^= 0b1100_0000,
            3 => codepoint ^= 0b1110_0000,
            4 => codepoint ^= 0b1111_0000,
            _ => return None,
        }
        if self.current_length != self.full_length {
            return None;
        }
        for i in 1..self.full_length {
            codepoint = (codepoint << 6) | ((self.bytes[i as usize] ^ 0b10_000000) as u32)
        }
        Some(codepoint)
    }
    pub fn is_valid_codepoint(&self) -> bool {
        let Some(codepoint) = self.get_codepoint() else {
            return false;
        };
        match self.full_length {
            1 => codepoint <= 0x7F,
            2 => codepoint > 0x7F && codepoint <= 0x7FF,
            3 => codepoint > 0x7FF && codepoint <= 0xFFFF,
            4 => codepoint > 0xFFFF && codepoint <= 0x10FFFF,
            _ => unreachable!(),
        }
    }

    pub const fn current_len(&self) -> usize {
        self.current_length as usize
    }
    pub const fn full_len(&self) -> usize {
        self.full_length as usize
    }
}
