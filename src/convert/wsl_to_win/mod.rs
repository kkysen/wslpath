use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::ffi::OsStringExt;
use std::path::Path;

use thiserror::Error;

use crate::convert::wsl_to_win::init::{ConvertOptionsError, Root, WslPathError};
pub use crate::convert::wsl_to_win::init::Options;

mod init;
mod encode;

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error(transparent)]
    WslPath(#[from] WslPathError),
}

pub struct Converter {
    options: Options,
    root: Root,
}

impl Converter {
    fn convert_absolute_path_into_buf(&self, _path: &mut [u8], _buf: &mut Vec<u8>) -> Result<(), ConvertError> {
        Ok(())
    }
}

impl super::Converter for Converter {
    type Options = Options;
    type OptionsError = ConvertOptionsError;
    type Error = ConvertError;
    
    fn new(mut options: Self::Options) -> Result<Self, Self::OptionsError> {
        options.init()?;
        let root = Root::new(&options)?;
        Ok(Self {
            options,
            root,
        })
    }
    
    fn convert_into_buf(&self, path_bytes: &mut [u8], buf: &mut Vec<u8>) -> Result<(), Self::Error> {
        let path = Path::new(OsStr::from_bytes(path_bytes.as_ref()));
        if self.options.canonicalize {
            let path = path.canonicalize()
                .map_err(WslPathError::Canonicalization)?;
            let mut path = path
                .into_os_string()
                .into_vec();
            self.convert_absolute_path_into_buf(path.as_mut_slice(), buf)?;
        } else {
            if !path.is_absolute() {
                Err(WslPathError::NotAbsolute)?;
                self.convert_absolute_path_into_buf(path_bytes, buf)?;
            }
        }
        Ok(())
    }
}
