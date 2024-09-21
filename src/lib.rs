use std::{
    collections::HashMap,
    error::Error,
    ffi::OsString,
    fs::File,
    io::{BufReader, Error as IOError, ErrorKind},
    path::*,
    vec::Vec,
};

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
        eprintln!("Invalid number of arguments");
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
            Ok(data) => {data}
            Err(error) => {
                file_states.insert(path, Err(error));
                continue;
            }
        };
        files.push((path, BufReader::new(file)));
    }
    todo!();
    Ok(())
}
