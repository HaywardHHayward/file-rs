use crate::vle::VariableLengthEncoding;

pub(crate) struct GbSequence {
    data: [u8; 4],
    current_length: u8,
    is_complete: bool,
}

impl VariableLengthEncoding for GbSequence {
    type Point = u8;

    #[inline]
    fn build(byte: Self::Point) -> Option<Self> {
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

    #[inline]
    fn is_complete(&self) -> bool {
        self.is_complete
    }

    #[inline]
    fn add_point(&mut self, point: Self::Point) -> bool {
        self.data[self.current_length as usize] = point;
        self.current_length += 1;
        if self.current_length == 2 {
            if (0x81..=0xFE).contains(&self.data[0]) {
                if (0x40..=0xFE).contains(&point) && point != 0x7F {
                    self.is_complete = true;
                    true
                } else {
                    ((0x81..=0x84).contains(&self.data[0]) || (0x90..=0xE3).contains(&self.data[0]))
                        && (0x30..=0x39).contains(&point)
                }
            } else {
                false
            }
        } else if self.current_length == 3 {
            (0x81..=0xFE).contains(&point)
        } else if self.current_length == 4 {
            self.is_complete = true;
            (0x30..=0x39).contains(&point)
        } else {
            false
        }
    }

    #[inline]
    fn is_valid(&self) -> bool {
        if self.is_complete && self.current_length == 1 {
            matches!(self.data[0], 0x08..=0x0D | 0x1B | 0x20..=0x7E)
        } else {
            self.is_complete
        }
    }
}
