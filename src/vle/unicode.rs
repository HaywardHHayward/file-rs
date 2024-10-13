pub(crate) mod utf16sequence;
pub(crate) mod utf8sequence;
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
