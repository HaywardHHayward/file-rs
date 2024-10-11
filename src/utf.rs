pub(crate) mod utf16sequence;
pub(crate) mod utf8sequence;

pub(crate) trait Utf {
    type Point;
    fn get_codepoint(&self) -> u32;
    fn add_point(&mut self, point: Self::Point) -> bool;
    fn is_valid(&self) -> bool {
        is_valid_codepoint(self.get_codepoint())
    }
}
pub(crate) const fn is_valid_codepoint(codepoint: u32) -> bool {
    char::from_u32(codepoint).is_some()
}
pub(crate) const fn is_text(codepoint: u32) -> bool {
    if char::from_u32(codepoint).is_none() {
        return false;
    }
    !((codepoint < 0xFF)
        && !(0x08 <= codepoint && 0x0D >= codepoint)
        && codepoint != 0x1B
        && !(0x20 <= codepoint && 0x7E >= codepoint)
        && 0xA0 > codepoint)
}

#[derive(Copy, Clone)]
pub(crate) enum Endianness {
    BigEndian,
    LittleEndian,
}
