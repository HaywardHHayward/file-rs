use crate::utf::*;

#[derive(Clone, Copy)]
enum Utf16Data {
    Bmp([u8; 2]),
    Surrogate([u8; 4]),
}

pub(crate) struct Utf16Sequence {
    data: Utf16Data,
    endianness: Endianness,
}

impl Utf for Utf16Sequence {
    type Point = [u8; 2];

    #[inline]
    fn get_codepoint(&self) -> u32 {
        match self.data {
            Utf16Data::Bmp(bytes) => Self::bytes_to_u16(bytes, self.endianness) as u32,
            Utf16Data::Surrogate(bytes) => {
                let high = Self::bytes_to_u16([bytes[0], bytes[1]], self.endianness) as u32;
                let low = Self::bytes_to_u16([bytes[2], bytes[3]], self.endianness) as u32;
                ((high - 0xD800) * 0x400) + (low - 0xDC00) + 0x10000
            }
        }
    }
    #[inline]
    fn add_point(&mut self, point: Self::Point) -> bool {
        match self.data {
            Utf16Data::Bmp(_) => false,
            Utf16Data::Surrogate(ref mut bytes) => {
                if !(0xDC00..=0xDFFF).contains(&Self::bytes_to_u16(point, self.endianness)) {
                    false
                } else {
                    [bytes[2], bytes[3]] = [point[0], point[1]];
                    true
                }
            }
        }
    }
    #[inline]
    fn is_valid(&self) -> bool {
        let value = self.get_codepoint();
        match self.data {
            Utf16Data::Bmp(_) => {
                matches!(value, 0x0000..=0xD7FF | 0xE000..=0xFFFF) && is_valid_codepoint(value)
            }
            Utf16Data::Surrogate(_) => {
                matches!(value, 0x010000..=0x10FFFF) && is_valid_codepoint(value)
            }
        }
    }
}

impl Utf16Sequence {
    #[inline]
    pub(crate) const fn new(bytes: [u8; 2], endianness: Endianness) -> Self {
        let data = if matches!(Self::bytes_to_u16(bytes, endianness), 0xD800..=0xDBFF) {
            Utf16Data::Surrogate([bytes[0], bytes[1], 0, 0])
        } else {
            Utf16Data::Bmp([bytes[0], bytes[1]])
        };
        Self { data, endianness }
    }
    #[inline]
    pub(crate) const fn bytes_to_u16(bytes: [u8; 2], endianness: Endianness) -> u16 {
        match endianness {
            Endianness::BigEndian => u16::from_be_bytes([bytes[0], bytes[1]]),
            Endianness::LittleEndian => u16::from_le_bytes([bytes[0], bytes[1]]),
        }
    }
    pub(crate) const fn is_surrogate(&self) -> bool {
        matches!(self.data, Utf16Data::Surrogate(_))
    }
}
