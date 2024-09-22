use file::file;

fn main() -> Result<(), std::io::Error> {
    let results = file();
    if results.is_err() {
        eprintln!("Usage: file [files]");
    }
    results
}
