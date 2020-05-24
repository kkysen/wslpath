mod init;
mod decode;

use crate::convert::{WindowsPathSep, IllegalWindowsFileNameCharError};
use std::ffi::{OsString, OsStr};
use std::path::PathBuf;
use thiserror::Error;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::ffi::OsStrExt;
pub use crate::convert::win_to_wsl::init::Options;
use crate::convert::win_to_wsl::init::{Root, ConvertOptionsError};

pub struct Converter {
    options: Options,
    root: Root,
}

impl WindowsPathSep {
    /// convert path sep to to posix path sep
    /// if we're accepting / as the Windows sep, then do nothing
    /// if we're accepting \ as the Windows sep, need to replace them w/ /
    fn convert_sep(&self, path: &mut [u8]) {
        use WindowsPathSep::*;
        match self {
            Slash => {} // already all /,
            BackSlash => {
                for c in path {
                    if *c == BackSlash.into() {
                        *c = Slash.into();
                    }
                }
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("parse error")]
    Parse,
    #[error(transparent)]
    IllegalFileNameChar(#[from] IllegalWindowsFileNameCharError),
}

impl Converter {
    fn try_fix_root_loop<'a>(&self, path: &'a [u8]) -> Option<&'a [u8]> {
        let root = self
            .root
            .windows_store
            .as_ref()?
            .as_os_str()
            .as_bytes();
        if path.len() < root.len() {
            return None;
        }
        if &path[..root.len()] != root {
            return None;
        }
        Some(&path[root.len()..])
    }
    
    /// fix potential root loops,
    /// i.e., the loop from / to windows store root,
    /// by just removing that redundant prefix
    fn fix_root_loop<'a>(&self, path: &'a [u8]) -> &'a [u8] {
        self.try_fix_root_loop(path)
            .unwrap_or(path)
    }
    
    /// do the actual path conversion,
    /// assuming the posix path sep / is used
    /// and not fixing root loops
    /// i.e., only convert prefix
    fn raw_convert(&self, path: &[u8], buf: &mut Vec<u8>) -> Result<(), ConvertError> {
        let verbatim = b"//?/";
        let path = if path.starts_with(verbatim) {
            &path[verbatim.len()..]
        } else {
            path
        };
        let unc_root = self.root.unc.as_bytes();
        match path {
            _ if path.starts_with(unc_root) => {
                let path = &path[unc_root.len()..];
                buf.reserve(path.len());
                decode::path(path, buf)?;
            }
            [drive, b':', b'/', path @ ..] => {
                let path = self.fix_root_loop(path);
                let len = path.len() + (b"/mnt/c".len() - b"C:".len());
                buf.reserve(len);
                buf.extend_from_slice(b"/mnt/");
                buf.push(drive.to_ascii_lowercase());
                buf.push(b'/');
                decode::path(path, buf)?;
            }
            _ => return Err(ConvertError::Parse),
        };
        Ok(())
    }
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
    
    fn convert_into_buf(&self, path: &mut [u8], buf: &mut Vec<u8>) -> Result<(), Self::Error> {
        self.options.sep.convert_sep(path);
        self.raw_convert(path, buf)?;
        Ok(())
    }
}
