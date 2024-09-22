mod utf8sequence;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Error as IOError, ErrorKind, Read, Seek, SeekFrom},
    path::*,
    sync::{Arc, Mutex}, thread,
    vec::Vec,
};

use utf8sequence::*;

#[derive(Debug)]
enum FileType {
    Empty,
    Ascii,
    Latin1,
    Utf8,
    Data,
}

type FileState = Result<FileType, IOError>;

pub fn file() -> Result<(), IOError> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() {
        eprintln!("Invalid number of arguments.");
        return Err(IOError::from(ErrorKind::InvalidInput));
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
        files.push((path.to_owned(), BufReader::new(file)));
    }
    if files.len() <= 1 {
        for (path, file) in files {
            file_states.insert(path, classify_file(file));
        }
    } else {
        let shared_map = Arc::new(Mutex::new(file_states));
        let mut thread_pool = Vec::with_capacity(files.len());
        for (path, file) in files {
            let map_copy = shared_map.clone();
            thread_pool.push(thread::spawn(move || {
                let data = classify_file(file);
                let mut map = map_copy.lock().unwrap();
                map.insert(path, data);
            }));
        }
        for thread in thread_pool {
            thread.join().unwrap();
        }
        file_states = Arc::try_unwrap(shared_map).unwrap().into_inner().unwrap();
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
    let [mut is_ascii, mut is_latin1, mut is_utf8] = [true; 3];
    let mut sequence_option: Option<Utf8Sequence> = None;
    let length = file.seek(SeekFrom::End(0))?;
    if length == 0 {
        return Ok(FileType::Empty);
    }
    file.rewind()?;
    let file_bytes = file.bytes();
    for result_byte in file_bytes {
        let byte = result_byte?;
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
            if is_utf8 {
                let sequence = sequence_option.as_ref().unwrap();
                if sequence.full_len() == sequence.current_len() {
                    if !sequence.is_valid_codepoint() {
                        is_utf8 = false;
                    }
                    sequence_option = None;
                }
            }
        }
        if !is_ascii && !is_latin1 && !is_utf8 {
            return Ok(FileType::Data);
        }
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
