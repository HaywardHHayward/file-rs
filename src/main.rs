use file::file;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    file(
        std::env::args_os()
            .map(std::ffi::OsString::from)
            .skip(1)
            .collect::<Vec<_>>(),
    )
}
