use crate::utf::*;

enum Utf8Type {
    Ascii([u8; 1]),
    Western([u8; 2]),
    Bmp([u8; 3]),
    Other([u8; 4]),
}

pub struct Utf8Sequence {
    utf8_type: Utf8Type,
    current_length: u8,
}

impl Utf for Utf8Sequence {
    type Point = u8;
    type Codepoint = u32;
    #[inline]
    fn get_codepoint(&self) -> Self::Codepoint {
        let mut codepoint: u32 = match self.utf8_type {
            Utf8Type::Ascii(value) => value[0],
            Utf8Type::Western(bytes) => bytes[0] ^ 0b1100_0000,
            Utf8Type::Bmp(bytes) => bytes[0] ^ 0b1110_0000,
            Utf8Type::Other(bytes) => bytes[0] ^ 0b1111_0000,
        } as u32;
        for i in 1..self.full_len() {
            codepoint = (codepoint << 6) | ((self.get(i).unwrap() ^ 0b10_000000) as u32);
        }
        codepoint
    }
    #[inline]
    fn add_point(&mut self, point: Self::Point) -> bool {
        if self.current_len() + 1 > self.full_len() {
            return false;
        }
        if point >= 0b1100_0000 || Self::is_invalid(point) {
            return false;
        }
        let reference = self.get_mut(self.current_len());
        if reference.is_none() {
            return false;
        }
        *reference.unwrap() = point;
        self.current_length += 1;
        true
    }
    #[inline]
    fn is_valid(&self) -> bool {
        let codepoint = self.get_codepoint();
        if !is_valid_codepoint(codepoint) {
            return false;
        }
        match self.utf8_type {
            Utf8Type::Ascii(_) => (..=0x7F).contains(&codepoint),
            Utf8Type::Western(_) => (0x80..=0x7FF).contains(&codepoint),
            Utf8Type::Bmp(_) => (0x800..=0xFFFF).contains(&codepoint),
            Utf8Type::Other(_) => (0x100000..=0x10FFFF).contains(&codepoint),
        }
    }
}

impl Utf8Sequence {
    pub const fn build(byte: u8) -> Option<Self> {
        if (byte >= 0x80 && byte <= 0xBF) || Self::is_invalid(byte) {
            return None;
        }
        let utf8_type = match byte.leading_ones() {
            0 => Utf8Type::Ascii([byte; 1]),
            2 => Utf8Type::Western([byte, 0]),
            3 => Utf8Type::Bmp([byte, 0, 0]),
            4 => Utf8Type::Other([byte, 0, 0, 0]),
            _ => return None,
        };
        Some(Self {
            utf8_type,
            current_length: 1,
        })
    }
    const fn get(&self, index: usize) -> Option<u8> {
        match self.utf8_type {
            Utf8Type::Ascii(value) => {
                if index < value.len() {
                    return Some(value[0]);
                }
                None
            }
            Utf8Type::Western(bytes) => {
                if index < bytes.len() {
                    return Some(bytes[index]);
                }
                None
            }
            Utf8Type::Bmp(bytes) => {
                if index < bytes.len() {
                    return Some(bytes[index]);
                }
                None
            }
            Utf8Type::Other(bytes) => {
                if index < bytes.len() {
                    return Some(bytes[index]);
                }
                None
            }
        }
    }
    fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        match self.utf8_type {
            Utf8Type::Ascii(ref mut value) => {
                if index < value.len() {
                    return Some(&mut value[0]);
                }
                None
            }
            Utf8Type::Western(ref mut bytes) => {
                if index < bytes.len() {
                    return Some(&mut bytes[index]);
                }
                None
            }
            Utf8Type::Bmp(ref mut bytes) => {
                if index < bytes.len() {
                    return Some(&mut bytes[index]);
                }
                None
            }
            Utf8Type::Other(ref mut bytes) => {
                if index < bytes.len() {
                    return Some(&mut bytes[index]);
                }
                None
            }
        }
    }
    pub const fn full_len(&self) -> usize {
        match self.utf8_type {
            Utf8Type::Ascii(v) => v.len(),
            Utf8Type::Western(v) => v.len(),
            Utf8Type::Bmp(v) => v.len(),
            Utf8Type::Other(v) => v.len(),
        }
    }
    const fn is_invalid(byte: u8) -> bool {
        matches!(byte, 0xC0 | 0xC1 | 0xF5..)
    }
    pub const fn current_len(&self) -> usize {
        self.current_length as usize
    }
}
