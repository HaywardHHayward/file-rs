use file::file;

fn main() {
    let results = file();
    if results.is_err() {
        eprintln!("{}. Usage: file [files]", results.unwrap_err());
    }
}
