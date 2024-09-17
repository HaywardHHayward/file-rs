pub fn file(argv: Vec<std::ffi::OsString>) -> Result<(), Box<dyn std::error::Error>> {
    if argv.is_empty() {
        if let Ok(exe) = std::env::current_exe() {
            eprintln!("Not enough arguments. Usage: {} [files]", exe.display());
        } else {
            eprintln!("Shiiiiiiits fucked man. Close Chrome or something before running again.")
        }
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput).into());
    }
    for argument in argv.iter() {
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
    if std::str::from_utf8(&file).is_ok() {
        return FileClassifications::Utf8;
    }
    if file.iter().all(|c| c.is_ascii() || *c >= 0xA0u8) {
        return FileClassifications::Latin1;
    }
    FileClassifications::Data
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use crate::file;

    #[test]
    fn no_args() {
        assert!(file(vec![]).is_err());
    }

    #[test]
    fn invalid_arg() {
        assert!(file(vec![OsString::from("foo")]).is_ok());
    }

    #[test]
    fn unreadable() {
        assert!(file(vec![OsString::from("./test_files/noread")]).is_ok());
    }

    #[test]
    fn test_data() {
        assert!(file(vec![OsString::from("./test_files/data.data")]).is_ok());
    }

    #[test]
    fn test_empty() {
        assert!(file(vec![OsString::from("./test_files/empty")]).is_ok());
    }

    #[test]
    fn test_iso() {
        assert!(file(vec![OsString::from("./test_files/iso8859-1.txt")]).is_ok());
    }

    #[test]
    fn test_ascii() {
        assert!(file(vec![OsString::from("./test_files/ascii.txt")]).is_ok());
    }

    #[test]
    fn test_utf8() {
        assert!(file(vec![OsString::from("./test_files/utf8.txt")]).is_ok());
    }

    #[test]
    fn test_all() {
        assert!(file(vec![
            OsString::from("foo"),
            OsString::from("./test_files/noread"),
            OsString::from("./test_files/data.data"),
            OsString::from("./test_files/empty"),
            OsString::from("./test_files/iso8859-1.txt"),
            OsString::from("./test_files/ascii.txt"),
            OsString::from("./test_files/utf8.txt")
        ])
        .is_ok());
    }
}
