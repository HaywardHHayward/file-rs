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
        self.data[self.current_length as usize] = codepoint;
        self.current_length += 1;
        if self.current_length == 2 {
            if (0x81..=0xFE).contains(&self.data[0]) {
                if (0x40..=0xFE).contains(&codepoint) && codepoint != 0x7F {
                    self.is_complete = true;
                    true
                } else {
                    ((0x81..=0x84).contains(&self.data[0]) || (0x90..=0xE3).contains(&self.data[0]))
                        && (0x30..=0x39).contains(&codepoint)
                }
            } else {
                false
            }
        } else if self.current_length == 3 {
            (0x81..=0xFE).contains(&codepoint)
        } else if self.current_length == 4 {
            self.is_complete = true;
            (0x30..=0x39).contains(&codepoint)
        } else {
            false
        }
    }
}
