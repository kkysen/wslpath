use thiserror::Error;

pub use crate::convert::wsl_to_win::init::Options;
use crate::convert::wsl_to_win::init::{ConvertOptionsError, Root};
use std::{fs, io};
use std::path::Path;
use std::ffi::OsStr;

mod init;
mod encode;

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("failed to canonicalize path")]
    CanonicalizationFailed(#[from] io::Error),
}

pub struct Converter {
    options: Options,
    root: Root,
}

impl super::Converter for Converter {
    type Options = Options;
    type OptionsError = ConvertOptionsError;
    type Error = ConvertError;
    
    fn new(options: Self::Options) -> Result<Self, Self::OptionsError> {
        let root = Root::new(&options)?;
        Ok(Self {
            options,
            root,
        })
    }
    
    fn convert_into_buf(&self, path: &mut [u8], _buf: &mut Vec<u8>) -> Result<(), Self::Error> {
        let _path = if self.options.canonicalize {
            Ok(path)
                .map(OsStr::from_bytes)
                .map(Path::new)
                .and_then(fs::canonicalize)?
                .into_os_string()
                .into_vec()
                .as_mut_slice()
        } else {
            path
        };
        unimplemented!()
    }
}
