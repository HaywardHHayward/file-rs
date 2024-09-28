pub trait Utf {
    type Point;
    type Codepoint;
    fn get_codepoint(&self) -> Self::Codepoint;
    fn add_point(&mut self, point: Self::Point) -> bool;
    fn is_valid(&self) -> bool;
}
pub const fn is_valid_codepoint(codepoint: u32) -> bool {
    char::from_u32(codepoint).is_some()
}
pub const fn is_text(codepoint: u32) -> bool {
    if char::from_u32(codepoint).is_none() {
        return false;
    }
    if (codepoint < 0xFF)
        && !(0x08 <= codepoint && 0x0D >= codepoint)
        && codepoint != 0x1B
        && !(0x20 <= codepoint && 0x7E >= codepoint)
        && 0xA0 > codepoint
    {
        return false;
    }
    true
}
