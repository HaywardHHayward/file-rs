mod vle;

use std::{
    cmp::min,
    collections::BTreeMap,
    ffi::OsString,
    fs::File,
    io::{prelude::*, BufReader, Error as IOError, ErrorKind},
    path::PathBuf,
    thread,
};

use itertools::Itertools;

use crate::vle::{
    gb_sequence::*,
    unicode::{utf16sequence::*, utf8sequence::*, *},
    VariableLengthEncoding,
};

pub enum BufferType {
    Empty,
    Ascii,
    Latin1,
    Utf8,
    Utf16,
    Gb,
    Data,
}

pub type BufferState = Result<BufferType, IOError>;

pub fn file(args: impl ExactSizeIterator<Item = OsString>) -> Result<(), IOError> {
    if args.len() == 0 {
        return Err(IOError::new(
            ErrorKind::InvalidInput,
            "Invalid number of arguments",
        ));
    }
    let shared_file_states = parking_lot::const_mutex(BTreeMap::new());
    thread::scope(|s| {
        for arg in args.unique_by(|a| std::fs::canonicalize(a).unwrap_or(PathBuf::from(a))) {
            // gets rid of duplicate file paths so we don't do work twice
            s.spawn(|| {
                let path = PathBuf::from(arg);
                let metadata = std::fs::metadata(&path);
                if let Err(error) = metadata {
                    let mut file_states = shared_file_states.lock();
                    file_states.insert(path, Err(error));
                    return;
                }
                let bytes = metadata.unwrap().len();
                if bytes == 0 {
                    let mut file_states = shared_file_states.lock();
                    file_states.insert(path, Ok(BufferType::Empty));
                    return;
                }
                let file = match File::open(&path) {
                    Ok(open_file) => open_file,
                    Err(error) => {
                        let mut file_states = shared_file_states.lock();
                        file_states.insert(path, Err(error));
                        return;
                    }
                };
                let reader = BufReader::with_capacity(min(8 * 1024, bytes as usize), file);
                let data = classify_file(reader);
                let mut file_states = shared_file_states.lock();
                file_states.insert(path, data);
            });
        }
    });
    let file_states = shared_file_states.into_inner();
    for (path, file_result) in file_states {
        let message = match file_result {
            Ok(file_type) => match file_type {
                BufferType::Empty => "empty",
                BufferType::Ascii => "ASCII text",
                BufferType::Latin1 => "ISO 8859-1 text",
                BufferType::Utf8 => "UTF-8 text",
                BufferType::Utf16 => "UTF-16 text",
                BufferType::Gb => "GB 18030 text",
                BufferType::Data => "data",
            },
            Err(error) => &error.to_string(),
        };
        println!("{}: {message}", path.display());
    }
    Ok(())
}

const fn is_byte_ascii(byte: u8) -> bool {
    matches!(byte, 0x08..=0x0D | 0x1B | 0x20..=0x7E)
}

const fn is_byte_latin1(byte: u8) -> bool {
    is_byte_ascii(byte) || byte >= 0xA0
}

pub fn classify_file(reader: impl Read) -> BufferState {
    let mut is_ascii = true;
    let mut is_latin1 = true;
    let [mut is_utf8, mut is_utf16] = [true; 2];
    let mut is_gb = true;
    let mut utf8_sequence: Option<Utf8Sequence> = None;
    let mut utf16_sequence: Option<Utf16Sequence> = None;
    let mut gb_sequence: Option<GbSequence> = None;
    let mut endianness: Option<Endianness> = None;
    let mut byte_buffer = [0; 2];
    let mut bytes_read = 0;
    for result_byte in reader.bytes() {
        let byte = result_byte?;
        bytes_read += 1;
        if is_ascii && !is_byte_ascii(byte) {
            is_ascii = false;
        }
        if !is_ascii && is_utf16 {
            byte_buffer[(bytes_read - 1) % 2] = byte;
            if bytes_read % 2 == 0 {
                if let Some(endian_value) = endianness {
                    let point = match endian_value {
                        Endianness::BigEndian => {
                            u16::from_be_bytes([byte_buffer[0], byte_buffer[1]])
                        }
                        Endianness::LittleEndian => {
                            u16::from_le_bytes([byte_buffer[0], byte_buffer[1]])
                        }
                    };
                    validate_vle(&mut is_utf16, &mut utf16_sequence, point);
                } else {
                    let be = u16::from_be_bytes(byte_buffer);
                    let le = u16::from_le_bytes(byte_buffer);
                    if be == 0xFEFF {
                        endianness = Some(Endianness::BigEndian);
                    } else if le == 0xFEFF {
                        endianness = Some(Endianness::LittleEndian);
                    } else {
                        is_utf16 = false;
                    }
                }
            }
        }
        if !is_ascii && is_utf8 {
            validate_vle(&mut is_utf8, &mut utf8_sequence, byte);
        }
        if !is_ascii && is_gb {
            validate_vle(&mut is_gb, &mut gb_sequence, byte);
        }
        if !is_ascii && is_latin1 && !is_byte_latin1(byte) {
            is_latin1 = false;
        }
        if !is_ascii && !is_utf16 && !is_utf8 && !is_gb && !is_latin1 {
            return Ok(BufferType::Data);
        }
    }
    if utf16_sequence.is_some() {
        is_utf16 = false;
    }
    if utf8_sequence.is_some() {
        is_utf8 = false;
    }
    if gb_sequence.is_some() {
        is_gb = false;
    }
    return match [is_ascii, is_utf16, is_utf8, is_latin1, is_gb] {
        [true, _, _, _, _] => Ok(BufferType::Ascii),
        [_, true, _, _, _] => Ok(BufferType::Utf16),
        [_, _, true, _, _] => Ok(BufferType::Utf8),
        [_, _, _, true, _] => Ok(BufferType::Latin1),
        [_, _, _, _, true] => Ok(BufferType::Gb),
        [_, _, _, _, _] => Ok(BufferType::Data),
    };

    #[inline]
    fn validate_vle<T: VariableLengthEncoding>(
        is_valid: &mut bool,
        vle_sequence: &mut Option<T>,
        point: T::Point,
    ) {
        if let Some(sequence) = vle_sequence.as_mut() {
            if !sequence.is_complete() && !sequence.add_point(point) {
                *is_valid = false;
                return;
            }
            if sequence.is_complete() {
                if !sequence.is_valid() {
                    *is_valid = false;
                }
                *vle_sequence = None;
            }
        } else if let Some(sequence) = T::build(point) {
            if !sequence.is_complete() {
                *vle_sequence = Some(sequence);
            } else if !sequence.is_valid() {
                *is_valid = false;
            }
        } else {
            *is_valid = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ascii() {
        let ascii: [&[u8]; 2] = [
            include_bytes!("../test_files/ascii.txt").as_slice(),
            include_bytes!("../test_files/harpers_ASCII.txt"),
        ];
        let result = ascii.map(|bytes| classify_file(BufReader::new(bytes)));
        assert!(result
            .iter()
            .all(|state| matches!(state, Ok(BufferType::Ascii))));
    }
    #[test]
    fn test_latin1() {
        let latin1: [&[u8]; 3] = [
            include_bytes!("../test_files/iso8859-1.txt"),
            include_bytes!("../test_files/die_ISO-8859-1.txt"),
            include_bytes!("../test_files/portugal_ISO-8859-1.txt"),
        ];
        let result = latin1.map(|bytes| classify_file(BufReader::new(bytes)));
        assert!(result
            .iter()
            .all(|state| matches!(state, Ok(BufferType::Latin1))))
    }
    #[test]
    fn test_utf8() {
        let utf8: [&[u8]; 3] = [
            include_bytes!("../test_files/utf8.txt"),
            include_bytes!("../test_files/utf8_test.txt"),
            include_bytes!("../test_files/shisei_UTF-8.txt"),
        ];
        let result = utf8.map(|bytes| classify_file(BufReader::new(bytes)));
        assert!(result
            .iter()
            .all(|state| matches!(state, Ok(BufferType::Utf8))));
    }
    #[test]
    fn test_utf16() {
        let utf16: [&[u8]; 4] = [
            include_bytes!("../test_files/le_utf16.txt"),
            include_bytes!("../test_files/be_utf16.txt"),
            include_bytes!("../test_files/shisei_UTF-16LE.txt"),
            include_bytes!("../test_files/shisei_UTF-16BE.txt"),
        ];
        let result = utf16.map(|bytes| classify_file(BufReader::new(bytes)));
        assert!(result
            .iter()
            .all(|state| matches!(state, Ok(BufferType::Utf16))));
    }
    #[test]
    fn test_data() {
        let data: &[u8] = include_bytes!("../test_files/data.data");
        assert!(matches!(
            classify_file(BufReader::new(data)),
            Ok(BufferType::Data)
        ));
    }
    #[test]
    fn test_gb() {
        let data: [&[u8]; 2] = [
            include_bytes!("../test_files/gb_test.txt"),
            include_bytes!("../test_files/gb.txt"),
        ];
        let result = data.map(|bytes| classify_file(BufReader::new(bytes)));
        assert!(result
            .iter()
            .all(|state| matches!(state, Ok(BufferType::Gb))));
    }
}
