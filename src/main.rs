use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::Path;

use itertools::{Either, Itertools};
use structopt::StructOpt;

use print_bytes::{eprint_bytes, print_bytes};
use wslpath::convert::{BulkConversion, Converter, PathSeparators, win_to_wsl, wsl_to_win};
use wslpath::convert::line_sep::LineSep;
use wslpath::convert::path_sep::WindowsPathSep;
use std::env;

#[derive(StructOpt, Debug)]
struct SharedArgs {
    #[structopt(long)]
    from_files: bool,
    #[structopt(long, default_value)]
    path_sep: WindowsPathSep,
    #[structopt(long, default_value)]
    read_line_sep: LineSep,
    #[structopt(long, default_value)]
    write_line_sep: LineSep,
    #[structopt(parse(from_os_str))]
    paths: Vec<OsString>,
}

#[derive(StructOpt, Debug)]
enum Args {
    Win {
        #[structopt(flatten)]
        args: SharedArgs,
        #[structopt(long)]
        dont_canonicalize: bool,
    },
    WSL {
        #[structopt(flatten)]
        args: SharedArgs,
        #[structopt(long)]
        dont_convert_root_loop: bool,
    },
}

fn print_converted<C: Converter>(
    converted: &BulkConversion<C>, source_path: Option<&Path>, line_sep: &LineSep,
) {
    let BulkConversion {
        paths,
        remainder_index: _,
        errors,
    } = converted;
    print_bytes(paths);
    if errors.is_empty() {
        return;
    }
    if let Some(source_path) = source_path {
        eprint_bytes(source_path);
        eprint!(":{}", line_sep.value());
    }
    for error in errors {
        eprint!("\t{:#?}{}", error, line_sep.value());
    }
}

fn run<C: Converter>(args: SharedArgs, converter: C) {
    if !args.from_files {
        let mut paths = args.paths
            .into_iter()
            .map(|it| it.into_vec())
            .intersperse(vec![0])
            .flat_map(|it| it.into_iter())
            .collect_vec();
        paths.push(0); // trailing delimiter
        let seps = PathSeparators {
            input: LineSep::Null,
            output: args.write_line_sep,
        };
        let converted = converter.convert_all(paths.as_mut_slice(), &seps);
        print_converted(&converted, Some(Path::new("args")), &args.write_line_sep);
    } else {
        let seps = PathSeparators {
            input: args.read_line_sep,
            output: args.write_line_sep,
        };
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
                    .map_err(|it| (path, it))
            })
            .partition_map(Either::from);
        if !errors.is_empty() {
            for (path, error) in errors {
                eprint!("{:#?}: {:#?}{}", path, error, args.write_line_sep.value());
            }
            return;
        }
        for (path, file) in files {
            for (i, converted) in file.enumerate() {
                let source_path = Some(path).filter(|_| i == 0);
                match converted {
                    Err(e) => {
                        eprint!("{:#?}: {:#?}{}", path, e, args.write_line_sep.value());
                        break;
                    }
                    Ok(converted) => {
                        print_converted(&converted, source_path, &args.write_line_sep);
                    }
                };
            }
        }
    }
}

#[paw::main]
fn main(args: Args) -> anyhow::Result<()> {
    eprintln!("{:#?}", args);
    use Args::*;
    match args {
        WSL { args, dont_convert_root_loop } => {
            use win_to_wsl::{Converter, Options};
            let options = Options {
                sep: args.path_sep,
                convert_root_loop: !dont_convert_root_loop,
            };
            run(args, Converter::new(options)?);
        }
        Win {args, dont_canonicalize} => {
            use wsl_to_win::{Converter, Options};
            let options = Options {
                sep: args.path_sep,
                canonicalize: !dont_canonicalize,
                base_directory: Some(env::current_dir()?),
            };
            run(args, Converter::new(options)?);
        }
    }
    Ok(())
}
