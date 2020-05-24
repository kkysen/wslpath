use std::ffi::OsString;
use structopt::StructOpt;
use wslpath::convert::{win_to_wsl, WindowsPathSep, Converter, wsl_to_win, InputPathSeparator, BulkConversion};
use print_bytes::println_bytes;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::{fmt, fs, io};
use wslpath::convert::path_sep::WindowsPathSep;
use wslpath::convert::line_sep::LineSep;
use itertools::{Itertools, Either};
use print_bytes::{print_bytes, eprintln_bytes};
use std::path::Path;
use std::fs::File;
use anyhow::bail;

#[derive(StructOpt, Debug)]
enum To {
    Win,
    WSL {
        #[structopt(long)]
        convert_root_loop: bool,
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
    #[structopt(long)]
    path_sep: WindowsPathSep,
    #[structopt(long)]
    read_line_sep: LineSep,
    #[structopt(long)]
    write_line_sep: LineSep,
}

struct ConvertAndPrint<'a, C: Converter> {
    converter: C,
    paths: &'a mut Vec<u8>,
    input_sep: &'a LineSep,
    output_sep: &'a LineSep,
    source_path: &'a Path,
}

impl<'a, C: Converter> ConvertAndPrint<'a, C> {
    fn run(&self) {
        let BulkConversion {
            paths,
            errors,
        } = self.converter.convert_all_flat(
            self.paths.as_mut_slice(),
            self.input_sep.sep(),
            self.output_sep.sep(),
        );
        print_bytes(paths);
        eprint_bytes(self.source_path);
        eprint!(":{}", self.output_sep.as_str());
        for error in errors {
            eprint!("\t{:#?}{}", error, self.output_sep.as_str());
        }
    }
}

fn run<C: Converter>(args: &Args, options: C::Options) -> anyhow::Result<()> {
    let converter = C::new(options)?;
    if !args.from_files {
        let mut paths = args.paths
            .into_iter()
            .map(|it| it.into_vec())
            .intersperse(vec![0])
            .flat_map(|it| it.into_iter())
            .collect();
        ConvertAndPrint {
            converter: &converter,
            paths: &mut paths,
            input_sep: &LineSep::Null,
            output_sep: &args.write_line_sep,
            source_path: "args".into(),
        }.run();
    } else {
        let stdin_path = "/proc/self/fd/0"; // /proc/self/fd/0 is more portable than /dev/stdin
        let (errors, files) = match args.paths.as_slice() {
            [] => &[stdin_path.into()],
            paths => paths,
        }.iter()
            .map(Path::new)
            .map(|path| (path, File::open(path)?))
            .partition_map::<Vec<io::Error>, Vec<(&Path, File)>>(Either::from);
        if !errors.is_empty() {
            bail!("io error"); // TODO print all errors
        }
        for (path, file) in files {
            // for now, read whole file at once
            // but for non-file files, I should buffer the conversion
            let mut paths = fs::read(file)?;
            ConvertAndPrint {
                converter: &converter,
                paths: &mut paths,
                input_sep: &args.read_line_sep,
                output_sep: &args.write_line_sep,
                source_path: path,
            }.run();
        }
    }
    Ok(())
}

#[paw::main]
fn main(args: Args) -> anyhow::Result<()> {
    println!("{:#?}", args);
    match args.to {
        To::WSL { convert_root_loop } => {
            run(&args, win_to_wsl::Options {
                convert_root_loop,
                sep: args.path_sep,
            })?;
        }
        To::Win => {
            run(&args, wsl_to_win::Options {
                sep: args.path_sep,
            })?;
        }
    }
    Ok(())
}
