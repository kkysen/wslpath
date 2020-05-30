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
    #[error("path contains a null byte at index {index}")]
    NullByte { index: usize },
}

pub struct Converter {
    options: Options,
    root: Root,
}

impl Converter {
    //
//
//     /// root contains the wsl unc prefix
//     /// prefix is the absolute prefix of the path, w/o a trailing /
//     /// path is the path relative to the prefix, w/o a leading /
//     /// even if the given path was absolute,
//     /// "/" is passed as the prefix and path is relativized
//     fn convert_raw(root: &Root, prefix: &[u8], path: &[u8], buf: &mut Vec<u8>) {
//         let mnt = b"/mnt/";
//         if prefix.starts_with(mnt) {
//             let prefix = &prefix[mnt.len()..];
//             if prefix.len() >= mnt.len() + 2 {
//                 prefix[mnt.len()]
//             } else {
//
//             }
//         }
//         match prefix {
//             mnt => {
//
//             },
//             b"/mnt/" => {
//                 // the trailing / here means the prefix contains the next path component
//
//             }
//             b"/" => {
//
//             },
//             b"/mnt" => {
//
//             },
//         }
//     }
//
    fn convert_raw(&self, _path: &mut [u8], _buf: &mut Vec<u8>) -> Result<(), ConvertError> {
        todo!()
    }
//
//     fn convert_absolute_path_into_buf(&self, path: &mut [u8], buf: &mut Vec<u8>) -> Result<(), ConvertError> {
//         if self.options.canonicalize {
//             let path = path.canonicalize()
//                 .map_err(WslPathError::Canonicalization)?;
//             let mut path = path
//                 .into_os_string()
//                 .into_vec();
//             self.convert_absolute_canonical_path_into_buf(path.as_mut_slice(), buf)?;
//         } else {
//             if !path.is_absolute() {
//                 Err(WslPathError::NotAbsolute)?;
//                 self.convert_absolute_canonical_path_into_buf(path_bytes, buf)?;
//             }
//         }
//         Ok(())
//     }
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
        for (i, c) in path_bytes.iter().enumerate() {
            if *c == 0 {
                return Err(ConvertError::NullByte { index: i });
            }
        }
        
        let path = Path::new(OsStr::from_bytes(path_bytes.as_ref()));
        
        // let convert_absolute = || -> Result<(), ConvertError> {
        //     self.convert_raw(path_bytes, buf)?;
        //     Ok(())
        // };
        //
        // let canonicalize_and_convert = || -> Result<(), ConvertError> {
        //     let path = path.canonicalize()
        //         .map_err(WslPathError::Canonicalization)?;
        //     let mut path = path
        //         .into_os_string()
        //         .into_vec();
        //     self.convert_raw(path.as_mut_slice(), buf)?;
        //     Ok(())
        // };
        //
        // if let Some(base_dir) = &self.options.base_directory {
        //     if path.is_absolute() {
        //         // if already absolute, don't relativize it
        //         if self.options.canonicalize {
        //             canonicalize_and_convert()?;
        //         } else {
        //             convert_absolute()?;
        //         }
        //     } else {
        //         let path = base_dir.join(path);
        //         if self.options.canonicalize {
        //             todo!()
        //         } else {
        //             todo!()
        //         }
        //     }
        // } else {
        //     if self.options.canonicalize {
        //         canonicalize_and_convert()?;
        //     } else {
        //         if !path.is_absolute() {
        //             Err(WslPathError::NotAbsolute)?;
        //         } else {
        //             convert_absolute()?;
        //         }
        //     }
        // }
        Ok(())
    }
}
