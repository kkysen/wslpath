pub mod win_to_wsl;
pub mod wsl_to_win;
mod wsl;
mod windows_file_name_char;
pub mod path_sep;
pub mod line_sep;

use thiserror::Error;
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::fmt;
use crate::convert::WindowsPathSep::BackSlash;
use crate::convert::win_to_wsl::Options;
use std::ffi::{OsStr, OsString};

pub trait InputPathSeparator {
    fn matches(&self, c: u8) -> bool;
}

pub trait OutputPathSeparator {
    fn write_to_buf(&self, buf: &mut Vec<u8>);
}

#[derive(Error, Debug)]
#[error("error convert {path:#?} at index {index}: {source:#?}")]
pub struct OneConvertError<'a, T: Converter> {
    index: usize,
    path: &'a [u8],
    source: Converter::Error,
}

pub struct BulkConversion<'a, T: Converter> {
    pub paths: OsString,
    pub errors: Vec<OneConvertError<'a, T>>,
}

pub trait Converter {
    type Options;
    type OptionsError;
    type Error;
    
    fn new(options: Self::Options) -> Result<Self, Self::OptionsError>;
    
    /// Lower-level version of [`convert`].
    /// Takes a [`&mut [u8]`] directly to avoid any copies and allow slices.
    /// Outputs into a [`Vec<u8>`] to avoid extra copies on return.
    /// This is useful for minimizing overhead on bulk conversions.
    fn convert_into_buf(&self, path: &mut [u8], buf: &mut Vec<u8>) -> Result<(), Self::Error>;
    
    /// Convert an absolute path.
    fn convert<S: AsRef<OsStr> + ?Sized>(&self, path: &S) -> Result<OsString, Self::Error> {
        let mut path = path
            .as_ref()
            .to_os_string()
            .into_vec();
        let mut buf = Vec::new();
        self.convert_into_buf(path.as_mut_slice(), &mut buf)?;
        Ok(buf)
            .map(OsString::from_vec)
    }
    
    fn convert_all<'a, I, OutputSep>(&self, paths: I, output_sep: OutputSep) -> BulkConversion<'a, Self>
        where I: Iterator<Item = &'a mut [u8]>,
              OutputSep: OutputPathSeparator {
        // at most one more allocation
        let mut buf = Vec::with_capacity(paths.len());
        let mut errors = Vec::new();
        for (index, path) in paths
            .enumerate() {
            let result = self.convert_into_buf(path, &mut buf);
            if let Err(source) = result {
                errors.push(OneConvertError {
                    index,
                    path,
                    source,
                });
            }
            output_sep.write_to_buf(&mut buf);
            buf.push(out_sep);
        }
        let paths = OsString::from_vec(buf);
        BulkConversion {
            paths,
            errors,
        }
    }
    
    fn convert_all_flat<'a, Sep>(&self, paths: &'a mut [u8], input_sep: InputSep, output_sep: OutputSep) -> BulkConversion<'a, Self>
        where InputSep: InputPathSeparator,
              OutputSep: OutputPathSeparator {
        let paths = paths
            .split_mut(|c| input_sep.matches(*c))
            .filter(|it| !it.is_empty());
        self.convert_all(paths, output_sep)
    }
}
