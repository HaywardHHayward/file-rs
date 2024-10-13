use crate::vle::{unicode::*, VariableLengthEncoding};

#[derive(Clone, Copy)]
enum Utf16Type {
    Bmp(u16),
    Surrogate { data: [u16; 2], is_complete: bool },
}

pub(crate) struct Utf16Sequence(Utf16Type);

impl VariableLengthEncoding for Utf16Sequence {
    type Point = u16;

    #[inline]
    fn build(point: Self::Point) -> Option<Self> {
        let data = if matches!(point, 0xD800..=0xDBFF) {
            Utf16Type::Surrogate {
                data: [point, 0],
                is_complete: false,
            }
        } else {
            Utf16Type::Bmp(point)
        };
        Some(Self(data))
    }

    #[inline]
    fn is_complete(&self) -> bool {
        match self.0 {
            Utf16Type::Bmp(_) => true,
            Utf16Type::Surrogate {
                data: _,
                is_complete,
            } => is_complete,
        }
    }

    #[inline]
    fn add_point(&mut self, point: Self::Point) -> bool {
        match self.0 {
            Utf16Type::Bmp(_) => false,
            Utf16Type::Surrogate {
                data: ref mut bytes,
                ref mut is_complete,
            } => {
                if !(0xDC00..=0xDFFF).contains(&point) {
                    false
                } else {
                    bytes[1] = point;
                    *is_complete = true;
                    true
                }
            }
        }
    }

    #[inline]
    fn is_valid(&self) -> bool {
        let value = self.get_codepoint();
        match self.0 {
            Utf16Type::Bmp(_) => {
                matches!(value, 0x0000..=0xD7FF | 0xE000..=0xFFFF) && is_text(value)
            }
            Utf16Type::Surrogate {
                data: _,
                is_complete,
            } => is_complete && matches!(value, 0x010000..=0x10FFFF) && is_text(value),
        }
    }
}

impl Utf16Sequence {
    #[inline]
    fn get_codepoint(&self) -> u32 {
        match self.0 {
            Utf16Type::Bmp(bytes) => bytes as u32,
            Utf16Type::Surrogate {
                data: bytes,
                is_complete: _,
            } => {
                let high = bytes[0] as u32;
                let low = bytes[1] as u32;
                ((high - 0xD800) * 0x400) + (low - 0xDC00) + 0x10000
            }
        }
    }
}
