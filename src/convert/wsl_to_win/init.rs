use crate::convert::path_sep::WindowsPathSep;
use crate::convert::wsl::get_unc_root;
use std::path::PathBuf;

pub struct Options {
    pub sep: WindowsPathSep,
    pub canonicalize: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            sep: WindowsPathSep::default(),
            canonicalize: true,
        }
    }
}

pub struct Root {
    pub unc: PathBuf,
}

#[derive(Error, Debug)]
pub enum ConvertOptionsError {
    #[error("not running on WSL")]
    NotWsl,
}

impl Root {
    pub fn new(_options: &Options) -> Result<Self, ConvertOptionsError> {
        Ok(Self {
            unc: get_unc_root()
                .map(PathBuf::from)
                .ok_or(ConvertOptionsError::NotWsl)?,
        })
    }
}

