#[derive(Copy, Clone)]
pub enum Endianness {
    BigEndian,
    LittleEndian,
}

pub enum Utf16Type {
    Surrogate(u32),
    Bmp(u16),
}

pub struct Utf16Sequence {
    bytes: [u8; 4],
    is_surrogate: bool,
    endianness: Endianness,
}

impl Utf16Sequence {
    pub fn build(bytes: [u8; 2], endianness: Endianness) -> Self {
        let is_surrogate = match endianness {
            Endianness::BigEndian => {
                let codepoint = u16::from_be_bytes([bytes[0], bytes[1]]);
                (0xD800..=0xDBFF).contains(&codepoint)
            }
            Endianness::LittleEndian => {
                let codepoint = u16::from_le_bytes([bytes[0], bytes[1]]);
                (0xD800..=0xDBFF).contains(&codepoint)
            }
        };
        Self {
            bytes: [bytes[0], bytes[1], 0, 0],
            is_surrogate,
            endianness,
        }
    }
    pub fn bytes_to_u16(bytes: [u8; 2], endianness: Endianness) -> u16 {
        match endianness {
            Endianness::BigEndian => u16::from_be_bytes([bytes[0], bytes[1]]),
            Endianness::LittleEndian => u16::from_le_bytes([bytes[0], bytes[1]]),
        }
    }
    pub fn add_bytes(&mut self, bytes: [u8; 2]) -> bool {
        if !self.is_surrogate {
            return false;
        }
        if !(0xDC00..=0xDFFF).contains(&Self::bytes_to_u16(bytes, self.endianness)) { 
            return false;
        }
        self.bytes[2] = bytes[0];
        self.bytes[3] = bytes[1];
        true
    }
    pub fn get_codepoint(&self) -> Utf16Type {
        let from_bytes = match self.endianness {
            Endianness::LittleEndian => u16::from_le_bytes,
            Endianness::BigEndian => u16::from_be_bytes,
        };
        if self.is_surrogate {
            let high = from_bytes([self.bytes[0], self.bytes[1]]) as u32;
            let low = from_bytes([self.bytes[2], self.bytes[3]]) as u32;
            return Utf16Type::Surrogate(((high - 0xD800) * 0x400) + (low - 0xDC00) + 0x10000);
        }
        Utf16Type::Bmp(from_bytes([self.bytes[0], self.bytes[1]]))
    }
    pub fn is_valid(&self) -> bool {
        match self.get_codepoint() {
            Utf16Type::Surrogate(value) => {
                (0x010000..=0x10FFFF).contains(&value)
            }
            Utf16Type::Bmp(value) => {
                (0x0000..=0xD7FF).contains(&value) || (0xE000..=0xFFFF).contains(&value)
            }
        }
    }
    pub fn is_surrogate(&self) -> bool {
        self.is_surrogate
    }
}
