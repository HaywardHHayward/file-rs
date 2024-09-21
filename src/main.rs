use file::file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let results = file();
    if results.is_err() { 
        eprintln!("Usage: file [files]");
    }
    results
}
