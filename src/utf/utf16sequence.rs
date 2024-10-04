use crate::utf::*;
pub struct Utf16Sequence {
    bytes: [u8; 4],
    is_surrogate: bool,
    endianness: Endianness,
}

impl Utf for Utf16Sequence {
    type Point = [u8; 2];

    #[inline]
    fn get_codepoint(&self) -> u32 {
        let high = match self.endianness {
            Endianness::BigEndian => u16::from_be_bytes([self.bytes[0], self.bytes[1]]),
            Endianness::LittleEndian => u16::from_le_bytes([self.bytes[0], self.bytes[1]]),
        } as u32;
        if self.is_surrogate {
            let low = match self.endianness {
                Endianness::BigEndian => u16::from_be_bytes([self.bytes[2], self.bytes[3]]),
                Endianness::LittleEndian => u16::from_le_bytes([self.bytes[2], self.bytes[3]]),
            } as u32;
            ((high - 0xD800) * 0x400) + (low - 0xDC00) + 0x10000
        } else {
            high
        }
    }
    #[inline]
    fn add_point(&mut self, point: Self::Point) -> bool {
        if !self.is_surrogate {
            return false;
        }
        if !(0xDC00..=0xDFFF).contains(&Self::bytes_to_u16(point, self.endianness)) {
            return false;
        }
        self.bytes[2] = point[0];
        self.bytes[3] = point[1];
        true
    }
    #[inline]
    fn is_valid(&self) -> bool {
        let value = self.get_codepoint();
        if self.is_surrogate {
            matches!(value, 0x010000..=0x10FFFF) && is_valid_codepoint(value)
        } else {
            matches!(value, 0x0000..=0xD7FF | 0xE000..=0xFFFF) && is_valid_codepoint(value)
        }
    }
}

impl Utf16Sequence {
    #[inline]
    pub const fn new(bytes: [u8; 2], endianness: Endianness) -> Self {
        let is_surrogate = match endianness {
            Endianness::BigEndian => {
                let codepoint = u16::from_be_bytes([bytes[0], bytes[1]]);
                matches!(codepoint, 0xD800..=0xDBFF)
            }
            Endianness::LittleEndian => {
                let codepoint = u16::from_le_bytes([bytes[0], bytes[1]]);
                matches!(codepoint, 0xD800..=0xDBFF)
            }
        };
        Self {
            bytes: [bytes[0], bytes[1], 0, 0],
            is_surrogate,
            endianness,
        }
    }
    #[inline]
    pub const fn bytes_to_u16(bytes: [u8; 2], endianness: Endianness) -> u16 {
        match endianness {
            Endianness::BigEndian => u16::from_be_bytes([bytes[0], bytes[1]]),
            Endianness::LittleEndian => u16::from_le_bytes([bytes[0], bytes[1]]),
        }
    }
    pub const fn is_surrogate(&self) -> bool {
        self.is_surrogate
    }
}
