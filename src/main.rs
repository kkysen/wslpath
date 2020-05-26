use std::ffi::OsString;
use std::path::Path;
use std::os::unix::ffi::OsStringExt;

use itertools::{Either, Itertools};
use structopt::StructOpt;

use print_bytes::{eprint_bytes, print_bytes};
use wslpath::convert::{BulkConversion, Converter, win_to_wsl, wsl_to_win, PathSeparators};
use wslpath::convert::line_sep::LineSep;
use wslpath::convert::path_sep::WindowsPathSep;
use wslpath::util::enum_arg::EnumArg;

#[derive(StructOpt, Debug)]
enum To {
    Win,
    WSL {
        #[structopt(long)]
        dont_convert_root_loop: bool,
    },
}

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(subcommand)]
    to: To,
    #[structopt(parse(from_os_str))]
    paths: Vec<OsString>,
    #[structopt(long)]
    from_files: bool,
    #[structopt(long, possible_values = &WindowsPathSep::str_variants(), case_insensitive = true)]
    path_sep: WindowsPathSep,
    #[structopt(long, possible_values = &LineSep::str_variants(), case_insensitive = true)]
    read_line_sep: LineSep,
    #[structopt(long, possible_values = &LineSep::str_variants(), case_insensitive = true)]
    write_line_sep: LineSep,
}

fn print_converted<C: Converter>(converted: &BulkConversion<C>, source_path: Option<&Path>, line_sep: &LineSep) {
    let BulkConversion {
        paths,
        remainder_index: _,
        errors,
    } = converted;
    print_bytes(paths);
    if errors.is_empty() {
        return;
    }
    source_path.map(eprint_bytes);
    eprint!(":{}", line_sep.value());
    for error in errors {
        eprint!("\t{:#?}{}", error, line_sep.value());
    }
}

fn run<C: Converter>(args: Args, converter: C) {
    let seps = PathSeparators {
        input: args.read_line_sep,
        output: args.write_line_sep,
    };
    if !args.from_files {
        let mut paths = args.paths
            .into_iter()
            .map(|it| it.into_vec())
            .intersperse(vec![0])
            .flat_map(|it| it.into_iter())
            .collect_vec();
        paths.push(0); // trailing delimiter
        let converted = converter.convert_all(paths.as_mut_slice(), &seps);
        print_converted(&converted, Some(Path::new("args")), &args.write_line_sep);
    } else {
        let stdin_path = "/proc/self/fd/0"; // /proc/self/fd/0 is more portable than /dev/stdin
        let default_paths = [stdin_path.into()];
        let (errors, files): (Vec<_>, Vec<_>) = match args.paths.as_slice() {
            [] => &default_paths,
            paths => paths,
        }.iter()
            .map(Path::new)
            .map(|path| {
                converter
                    .convert_file(path, &seps, Default::default())
                    .map(|it| (path, it))
            })
            .partition_map(Either::from);
        if !errors.is_empty() {
            for error in errors {
                eprint!("{:#?}{}", error, args.write_line_sep.value());
            }
            return
        }
        for (path, file) in files {
            for (i, converted) in file.enumerate() {
                let source_path = Some(path).filter(|_| i == 0);
                match converted {
                    Err(e) => eprint!("{:#?}{}", e, args.write_line_sep.value()),
                    Ok(converted) => print_converted(&converted, source_path, &args.write_line_sep),
                };
            }
        }
    }
}

#[paw::main]
fn main(args: Args) -> anyhow::Result<()> {
    println!("{:#?}", args);
    match args.to {
        To::WSL { dont_convert_root_loop } => {
            use win_to_wsl::{Converter, Options};
            let options = Options {
                convert_root_loop: !dont_convert_root_loop,
                sep: args.path_sep,
            };
            run(args, Converter::new(options)?);
        }
        To::Win => {
            use wsl_to_win::{Converter, Options};
            let options = Options {
                sep: args.path_sep,
            };
            run(args, Converter::new(options)?);
        }
    }
    Ok(())
}
