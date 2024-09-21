mod utf8sequence;

use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Error as IOError, ErrorKind},
    path::*,
    vec::Vec,
};

use utf8sequence::*;

enum FileType {
    Empty,
    Ascii,
    Latin1,
    Utf8,
    Data,
}

type FileState = Result<FileType, IOError>;

pub fn file() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() {
        eprintln!("Invalid number of arguments.");
        return Err(IOError::from(ErrorKind::InvalidInput).into());
    }
    let mut file_paths: Vec<PathBuf> = Vec::with_capacity(args.len());
    let mut file_states: HashMap<PathBuf, FileState> = HashMap::with_capacity(args.len());
    for arg in args {
        let path = Path::new(&arg);
        match path.try_exists() {
            Ok(result) => {
                if !result {
                    file_states.insert(path.to_owned(), Err(IOError::from(ErrorKind::NotFound)));
                    continue;
                }
            }
            Err(error) => {
                file_states.insert(path.to_owned(), Err(error));
                continue;
            }
        }
        if !path.is_file() {
            file_states.insert(path.to_owned(), Err(IOError::from(ErrorKind::NotFound)));
        }
        file_paths.push(path.to_owned());
    }
    let mut files: Vec<(PathBuf, BufReader<File>)> = Vec::with_capacity(file_paths.len());
    for path in file_paths {
        let possible_file = File::open(&path);
        let file = match possible_file {
            Ok(data) => data,
            Err(error) => {
                file_states.insert(path, Err(error));
                continue;
            }
        };
        files.push((path, BufReader::new(file)));
    }
    for (path, file) in files {
        file_states.insert(path, classify_file(file));
    }
    for (path, file_result) in file_states {
        print!("{}: ", path.display());
        let message = match file_result {
            Ok(file_type) => match file_type {
                FileType::Empty => String::from("empty"),
                FileType::Ascii => String::from("ASCII text"),
                FileType::Latin1 => String::from("ISO 8859-1 text"),
                FileType::Utf8 => String::from("UTF-8 text"),
                FileType::Data => String::from("data"),
            },
            Err(error) => error.to_string(),
        };
        println!("{}", message);
    }
    Ok(())
}

const fn is_byte_ascii(byte: u8) -> bool {
    (byte >= 0x07 && byte <= 0x0D) || byte == 0x1B || (byte >= 0x20 && byte <= 0x7E)
}

const fn is_byte_latin1(byte: u8) -> bool {
    is_byte_ascii(byte) || byte >= 0xA0
}

fn classify_file(mut file: BufReader<File>) -> FileState {
    let mut is_ascii = true;
    let mut is_latin1 = true;
    let mut is_utf8 = true;
    let mut sequence_option: Option<Utf8Sequence> = None;
    let mut buffer = file.fill_buf()?;
    if buffer.is_empty() {
        return Ok(FileType::Empty);
    }
    while !buffer.is_empty() {
        for &byte in buffer {
            if is_ascii && !is_byte_ascii(byte) {
                is_ascii = false;
            }
            if is_latin1 && !is_byte_latin1(byte) {
                is_latin1 = false;
            }
            if is_utf8 {
                if sequence_option.is_none() {
                    match Utf8Sequence::build(byte) {
                        None => is_utf8 = false,
                        Some(data) => sequence_option = Some(data),
                    }
                } else if let Some(data) = &mut sequence_option {
                    if data.current_len() < data.full_len() && !data.add_byte(byte) {
                        is_utf8 = false;
                    }
                }
                let Some(ref sequence) = sequence_option else {
                    unreachable!()
                };
                if is_utf8 && sequence.full_len() == sequence.current_len() {
                    if !sequence.is_valid_codepoint() {
                        is_utf8 = false;
                    }
                    sequence_option = None;
                }
            }
            if !is_ascii && !is_latin1 && !is_utf8 {
                return Ok(FileType::Data);
            }
        }
        let buffer_length = buffer.len();
        file.consume(buffer_length);
        buffer = file.fill_buf()?;
    }
    if is_utf8 && sequence_option.is_some() {
        is_utf8 = false;
    }
    if is_ascii {
        Ok(FileType::Ascii)
    } else if is_utf8 {
        Ok(FileType::Utf8)
    } else if is_latin1 {
        Ok(FileType::Latin1)
    } else {
        Ok(FileType::Data)
    }
}
