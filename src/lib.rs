pub fn file() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = std::env::args_os()
        .map(std::ffi::OsString::from)
        .collect::<Vec<_>>();
    if arguments.len() < 2 {
        if let Ok(exe) = std::env::current_exe() {
            eprintln!("Not enough arguments. Usage: {} [files]", exe.display());
        } else {
            eprintln!("Shiiiiiiits fucked man. Close Chrome or something before running again.")
        }
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput).into());
    }
    for argument in arguments {
        print!("{}: ", argument.to_string_lossy());
        let possible_file = std::fs::read(std::path::Path::new(&argument));
        match possible_file {
            Ok(file) => {
                let classification = classify_file(file);
                match classification {
                    FileClassifications::Empty => {
                        println!("empty")
                    }
                    FileClassifications::Ascii => {
                        println!("ASCII text")
                    }
                    FileClassifications::Latin1 => {
                        println!("ISO 8859-1 text")
                    }
                    FileClassifications::Utf8 => {
                        println!("UTF-8 text")
                    }
                    FileClassifications::Data => {
                        println!("data")
                    }
                }
            }
            Err(error) => {
                println!("{error}");
            }
        }
    }
    Ok(())
}

enum FileClassifications {
    Empty,
    Ascii,
    Latin1,
    Utf8,
    Data,
}

fn classify_file(file: Vec<u8>) -> FileClassifications {
    if file.is_empty() {
        return FileClassifications::Empty;
    }
    if file.is_ascii() {
        return FileClassifications::Ascii;
    }
    if file.iter().all(|c| c.is_ascii() || *c >= 0xA0u8) { 
        return FileClassifications::Latin1;
    }
    if std::str::from_utf8(&file).is_ok() {
        return FileClassifications::Utf8;
    }
    FileClassifications::Data
}
