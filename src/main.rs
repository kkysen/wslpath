use std::ffi::OsString;
use structopt::StructOpt;
use wslpath::convert::win_to_wsl;
use wslpath::convert::win_to_wsl::Options;
use std::error::Error;

// #[derive(StructOpt, Debug)]
// struct Args {
//     #[structopt(short, long)]
//     absolute: bool,
//     #[structopt(subcommand)]
//     path_type: Option<PathTypeArg>,
//     #[structopt(parse(from_os_str))]
//     paths: Vec<OsString>,
// }
//
// #[derive(StructOpt, Debug)]
// enum PathTypeArg {
//     WSL,
//     Windows,
//     WindowsForwardSlash,
// }
//
// impl From<PathTypeArg> for PathType {
//     fn from(path_type: PathTypeArg) -> Self {
//         match path_type {
//             PathTypeArg::WSL => PathType::WSL,
//             PathTypeArg::Windows => PathType::Windows,
//             PathTypeArg::WindowsForwardSlash => PathType::WindowsForwardSlash,
//         }
//     }
// }
//
// impl Args {
//     pub fn read_paths_from_stdin(&self) -> bool {
//         self.paths.is_empty()
//     }
// }
//
// fn run() -> Option<()> {
//     let path = std::env::args_os().nth(1)?;
//     let converter = win_to_wsl::Converter::new(Options {
//         ..Default::default()
//     }).ok()?;
//     let path = converter.convert(path).ok()?;
//     println!("{:?}", path);
//     Some(())
// }

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(parse(from_os_str))]
    paths: Vec<OsString>,
}

#[paw::main]
fn main(args: Args) -> Result<(), Box<dyn Error>> {
    println!("{:#?}", args);
    let converter = win_to_wsl::Converter::new(Options {
        ..Default::default()
    })?;
    for path in args.paths {
        let path = converter.convert(path)?;
        println!("{:#?}", path);
    }
    Ok(())
}
