mod utf16sequence;
mod utf8sequence;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Error as IOError, ErrorKind, Read},
    path::{Path, PathBuf},
    sync::Mutex,
    thread,
    vec::Vec,
};

use utf8sequence::*;

use crate::utf16sequence::{Endianness, Utf16Sequence};

enum FileType {
    Empty,
    Ascii,
    Latin1,
    Utf8,
    Utf16,
    Data,
}

type FileState = Result<FileType, IOError>;

pub fn file() -> Result<(), IOError> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() {
        return Err(IOError::new(
            ErrorKind::InvalidInput,
            "Invalid number of arguments",
        ));
    }
    let mut files: Vec<(PathBuf, BufReader<File>)> = Vec::with_capacity(args.len());
    let mut file_states: HashMap<PathBuf, FileState> = HashMap::with_capacity(args.len());
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
            file_states.insert(path.to_owned(), Ok(FileType::Empty));
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
                FileType::Empty => "empty",
                FileType::Ascii => "ASCII text",
                FileType::Latin1 => "ISO 8859-1 text",
                FileType::Utf8 => "UTF-8 text",
                FileType::Utf16 => "UTF-16 text",
                FileType::Data => "data",
            },
            Err(error) => &error.to_string(),
        };
        println!("{message}");
    }
    Ok(())
}

const fn is_byte_ascii(byte: u8) -> bool {
    matches!(byte, 0x07..=0x0D | 0x1B | 0x20..=0x7E)
}

const fn is_byte_latin1(byte: u8) -> bool {
    is_byte_ascii(byte) || byte >= 0xA0
}

fn classify_file(file: BufReader<File>) -> FileState {
    let [mut is_ascii, mut is_latin1, mut is_utf8, mut is_utf16] = [true; 4];
    let mut utf8_sequence: Option<Utf8Sequence> = None;
    let mut utf16_sequence: Option<Utf16Sequence> = None;
    let mut endianness = Endianness::LittleEndian;
    let mut utf16_buffer: [u8; 2] = [0, 0];
    let mut bytes_read: usize = 0;
    let file_bytes = file.bytes();
    for result_byte in file_bytes {
        let byte = result_byte?;
        if is_utf16 {
            if bytes_read < 2 {
                utf16_buffer[bytes_read] = byte;
                bytes_read += 1;
                if bytes_read == 2 {
                    set_endianness(&mut utf16_sequence, &mut endianness, utf16_buffer);
                }
            } else if bytes_read % 2 < 2 {
                utf16_buffer[bytes_read % 2] = byte;
                bytes_read += 1;
                if bytes_read % 2 == 0 {
                    validate_utf16_sequence(
                        &mut is_utf16,
                        &mut utf16_sequence,
                        endianness,
                        utf16_buffer,
                    );
                }
            }
        }
        if is_ascii && !is_byte_ascii(byte) {
            is_ascii = false;
        }
        if !is_ascii && is_latin1 && !is_byte_latin1(byte) {
            is_latin1 = false;
        }
        if !is_ascii && is_utf8 {
            if utf8_sequence.is_none() {
                match Utf8Sequence::build(byte) {
                    None => {
                        is_utf8 = false;
                        continue;
                    }
                    Some(data) => utf8_sequence = Some(data),
                }
            } else {
                let data = utf8_sequence.as_mut().unwrap();
                if data.current_len() < data.full_len() && !data.add_byte(byte) {
                    is_utf8 = false;
                    continue;
                }
            }
            let sequence = utf8_sequence.as_ref().unwrap();
            if sequence.full_len() == sequence.current_len() {
                if !sequence.is_valid_codepoint() {
                    is_utf8 = false;
                }
                utf8_sequence = None;
            }
        }
        if !is_ascii && !is_utf8 && !is_latin1 && !is_utf16 {
            return Ok(FileType::Data);
        }
    }
    if is_ascii {
        return Ok(FileType::Ascii);
    }
    if utf8_sequence.is_some() {
        is_utf8 = false;
    }
    if is_utf8 {
        return Ok(FileType::Utf8);
    }
    if utf16_sequence.is_some() {
        is_utf16 = false;
    }
    if is_utf16 {
        return Ok(FileType::Utf16);
    }
    if is_latin1 {
        return Ok(FileType::Latin1);
    }
    return Ok(FileType::Data);

    fn set_endianness(
        utf16_sequence: &mut Option<Utf16Sequence>,
        endianness: &mut Endianness,
        utf16_buffer: [u8; 2],
    ) {
        let be = u16::from_be_bytes(utf16_buffer);
        let le = u16::from_le_bytes(utf16_buffer);
        if be == 0xFEFF {
            *endianness = Endianness::BigEndian
        } else if le == 0xFEFF {
            *endianness = Endianness::LittleEndian
        } else {
            *endianness = Endianness::LittleEndian;
            *utf16_sequence = Some(Utf16Sequence::build(utf16_buffer, *endianness));
        }
    }

    fn validate_utf16_sequence(
        is_utf16: &mut bool,
        utf16_sequence: &mut Option<Utf16Sequence>,
        endianness: Endianness,
        utf16_buffer: [u8; 2],
    ) {
        if let Some(ref mut sequence) = utf16_sequence {
            if !sequence.add_bytes(utf16_buffer) {
                *is_utf16 = false;
            }
            *utf16_sequence = None;
        } else {
            *utf16_sequence = Some(Utf16Sequence::build(utf16_buffer, endianness));
            if !utf16_sequence.as_ref().unwrap().is_surrogate() {
                *is_utf16 = utf16_sequence.as_ref().unwrap().is_valid();
                *utf16_sequence = None;
            }
        }
    }
}
