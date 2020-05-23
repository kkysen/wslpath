mod decode;
mod init;

use crate::convert::{Slash, IllegalWindowsFileNameCharError};
use std::ffi::OsString;
use std::path::PathBuf;
use thiserror::Error;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::ffi::OsStrExt;
pub use crate::convert::win_to_wsl::init::Options;
use crate::convert::win_to_wsl::init::Root;

pub struct Converter {
    options: Options,
    root: Root,
}

impl Slash {
    /// convert path sep to to posix path sep
    /// if we're accepting / as the Windows sep, then do nothing
    /// if we're accepting \ as the Windows sep, need to replace them w/ /
    fn convert_sep(&self, path: &mut [u8]) {
        use Slash::*;
        match self {
            Forward => {} // already all /,
            Backward => {
                for c in path {
                    if *c == Backward.value() {
                        *c = Forward.value();
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
    
    pub fn convert_into_buf(&self, path: OsString, buf: &mut Vec<u8>) -> Result<(), ConvertError> {
        let mut path = path.into_vec();
        self.options.sep.convert_sep(path.as_mut_slice());
        self.raw_convert(path.as_slice(), buf)?;
        Ok(())
    }
    
    /// Convert an absolute Windows path to an absolute WSL path.
    ///
    /// The Windows path is not checked for most errors, like invalid characters.
    ///
    /// ConvertError::Parse is only returned when the path can't be converted to an WSL path.
    ///
    /// The Windows path is assumed to use [`self.options.windows_slash`] as its separator.
    ///
    /// An [`OsString`] is taken instead of an [`&OsStr`] b/c it may be modified in-place,
    /// so this can eliminate an unnecessary copy.
    pub fn convert(&self, path: OsString) -> Result<PathBuf, ConvertError> {
        let mut buf = Vec::new();
        self.convert_into_buf(path, &mut buf)?;
        Ok(buf)
            .map(OsString::from_vec)
            .map(PathBuf::from)
    }
}
