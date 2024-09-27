mod utf;
mod utf16sequence;
mod utf8sequence;

use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::File,
    io::{prelude::*, BufReader, Error as IOError, ErrorKind},
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
    vec::Vec,
};

use crate::{utf::*, utf16sequence::*, utf8sequence::*};

enum BufferType {
    Empty,
    Ascii,
    Latin1,
    Utf8,
    Utf16,
    Data,
}
type BufferState = Result<BufferType, IOError>;

pub fn file() -> Result<(), IOError> {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    if args.is_empty() {
        return Err(IOError::new(
            ErrorKind::InvalidInput,
            "Invalid number of arguments",
        ));
    }
    let mut files: Vec<(PathBuf, BufReader<File>)> = Vec::with_capacity(args.len());
    let mut file_states: BTreeMap<PathBuf, BufferState> = BTreeMap::new();
    for arg in args {
        let path = Path::new(&arg);
        let file = match File::open(path) {
            Ok(data) => data,
            Err(error) => {
                file_states.insert(path.to_owned(), Err(error));
                continue;
            }
        };
        let metadata = match std::fs::metadata(path) {
            Ok(data) => data,
            Err(error) => {
                file_states.insert(path.to_owned(), Err(error));
                continue;
            }
        };
        if metadata.len() == 0 {
            file_states.insert(path.to_owned(), Ok(BufferType::Empty));
            continue;
        }
        files.push((path.to_owned(), BufReader::new(file)));
    }
    let shared_map = Mutex::new(file_states);
    thread::scope(|s| {
        for (path, file) in files {
            s.spawn(|| {
                let data = classify_file(file);
                let mut locked_map = shared_map.lock().unwrap();
                locked_map.insert(path, data);
            });
        }
    });
    file_states = shared_map.into_inner().unwrap();
    for (path, file_result) in file_states {
        print!("{}: ", path.display());
        let message = match file_result {
            Ok(file_type) => match file_type {
                BufferType::Empty => "empty",
                BufferType::Ascii => "ASCII text",
                BufferType::Latin1 => "ISO 8859-1 text",
                BufferType::Utf8 => "UTF-8 text",
                BufferType::Utf16 => "UTF-16 text",
                BufferType::Data => "data",
            },
            Err(error) => &error.to_string(),
        };
        println!("{message}");
    }
    Ok(())
}

const fn is_byte_ascii(byte: u8) -> bool {
    matches!(byte, 0x08..=0x0D | 0x1B | 0x20..=0x7E)
}

const fn is_byte_latin1(byte: u8) -> bool {
    is_byte_ascii(byte) || byte >= 0xA0
}

fn classify_file<T: Read>(reader: BufReader<T>) -> BufferState {
    let [mut is_ascii, mut is_latin1, mut is_utf8, mut is_utf16] = [true; 4];
    let mut utf8_sequence: Option<Utf8Sequence> = None;
    let mut utf16_sequence: Option<Utf16Sequence> = None;
    let mut endianness = Endianness::LittleEndian;
    let mut utf16_buffer: [u8; 2] = [0, 0];
    let mut bytes_read = 0;
    let reader_bytes = reader.bytes();
    for result_byte in reader_bytes {
        let byte = result_byte?;
        if is_utf16 {
            utf16_buffer[bytes_read % 2] = byte;
            bytes_read += 1;
            if bytes_read % 2 == 0 {
                validate_utf16(
                    &mut is_utf16,
                    &mut utf16_sequence,
                    &mut endianness,
                    utf16_buffer,
                    bytes_read,
                );
            }
        }
        if is_ascii && !is_byte_ascii(byte) {
            is_ascii = false;
        }
        if !is_ascii && is_latin1 && !is_byte_latin1(byte) {
            is_latin1 = false;
        }
        if !is_ascii && is_utf8 {
            validate_utf8(&mut is_utf8, &mut utf8_sequence, byte);
        }
        if !is_ascii && !is_utf16 && !is_utf8 && !is_latin1 {
            return Ok(BufferType::Data);
        }
    }
    if utf16_sequence.is_some() {
        is_utf16 = false;
    }
    if utf8_sequence.is_some() {
        is_utf8 = false;
    }
    return match [is_ascii, is_utf16, is_utf8, is_latin1] {
        [true, _, _, _] => Ok(BufferType::Ascii),
        [_, true, _, _] => Ok(BufferType::Utf16),
        [_, _, true, _] => Ok(BufferType::Utf8),
        [_, _, _, true] => Ok(BufferType::Latin1),
        [_, _, _, _] => Ok(BufferType::Data),
    };

    #[inline]
    fn validate_utf8(is_utf8: &mut bool, utf8_sequence: &mut Option<Utf8Sequence>, byte: u8) {
        if utf8_sequence.is_none() {
            match Utf8Sequence::build(byte) {
                None => {
                    *is_utf8 = false;
                    return;
                }
                Some(data) => *utf8_sequence = Some(data),
            }
        } else {
            let data = utf8_sequence.as_mut().unwrap();
            if data.current_len() < data.full_len() && !data.add_point(byte) {
                *is_utf8 = false;
                return;
            }
        }
        let sequence = utf8_sequence.as_ref().unwrap();
        if sequence.full_len() == sequence.current_len() {
            if !sequence.is_valid() || !is_text(sequence.get_codepoint()) {
                *is_utf8 = false;
            }
            *utf8_sequence = None;
        }
    }
    #[inline]
    fn validate_utf16(
        is_utf16: &mut bool,
        utf16_sequence: &mut Option<Utf16Sequence>,
        endianness: &mut Endianness,
        utf16_buffer: [u8; 2],
        bytes_read: usize,
    ) {
        if bytes_read == 2 {
            let be = u16::from_be_bytes(utf16_buffer);
            let le = u16::from_le_bytes(utf16_buffer);
            if be == 0xFEFF {
                *endianness = Endianness::BigEndian;
            } else if le == 0xFEFF {
                *endianness = Endianness::LittleEndian;
            } else {
                *is_utf16 = false;
            }
        } else {
            validate_utf16_sequence(is_utf16, utf16_sequence, *endianness, utf16_buffer);
        }
        #[inline]
        fn validate_utf16_sequence(
            is_utf16: &mut bool,
            utf16_sequence: &mut Option<Utf16Sequence>,
            endianness: Endianness,
            utf16_buffer: [u8; 2],
        ) {
            if let Some(sequence) = utf16_sequence.as_mut() {
                if !sequence.add_point(utf16_buffer) {
                    *is_utf16 = false;
                }
                *utf16_sequence = None;
            } else {
                *utf16_sequence = Some(Utf16Sequence::new(utf16_buffer, endianness));
                let sequence = utf16_sequence.as_ref().unwrap();
                if !sequence.is_surrogate() {
                    *is_utf16 = sequence.is_valid() && is_text(sequence.get_codepoint().into());
                    *utf16_sequence = None;
                }
            }
        }
    }
}
