pub struct GbSequence {
    data: [u8; 4],
    current_length: u8,
    is_complete: bool,
}

impl GbSequence {
    pub const fn build(byte: u8) -> Option<Self> {
        if byte == 0x80 || byte == 0xFF {
            return None;
        }
        let is_complete = byte <= 0x7F;
        Some(Self {
            data: [byte, 0, 0, 0],
            current_length: 1,
            is_complete,
        })
    }

    pub const fn is_complete(&self) -> bool {
        self.is_complete
    }

    #[inline]
    pub fn add_codepoint(&mut self, codepoint: u8) -> bool {
        if self.current_length == 1 {
            if self.data[0] <= 0x7F {
                false
            } else if 0x81 <= self.data[0] && self.data[0] <= 0xFE {
                match codepoint {
                    0x30..=0x39 => true,
                    0x40..=0x7E | 0x80..=0xFE => {
                        self.is_complete = true;
                        true
                    }
                    _ => false,
                }
            } else {
                return false;
            }
        } else if self.current_length == 2 {
            return 0x30 <= self.data[1]
                && self.data[1] <= 0x39
                && (0x81..=0xFE).contains(&codepoint);
        } else if self.current_length == 3 {
            return if (0x30..=0x39).contains(&codepoint) {
                self.is_complete = true;
                true
            } else {
                false
            };
        } else {
            false
        }
    }
}
