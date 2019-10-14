use wslpath::wsl_to_windows_path;
use std::path::Path;

fn run() -> Option<()> {
    let arg = std::env::args().nth(1)?;
    let path = Path::new(arg.as_str());
    let path = wsl_to_windows_path(path).ok()?;
    let path = path.expect("not on Windows");
    let path = path.to_str()?;
    println!("{}", path);
    Some(())
}

fn main() {
    run().unwrap();
}
