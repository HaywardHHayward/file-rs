use file::*;

fn main() {
    let results = file(std::env::args_os().skip(1));
    if results.is_err() {
        eprintln!("{}. Usage: file [files]", results.unwrap_err());
    }
}
