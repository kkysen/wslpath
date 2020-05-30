use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::convert::path_sep::WindowsPathSep;
use crate::convert::wsl::{get_unc_root, NotWslError, DrvFsMountPoint, get_drvfs_mount_points};

pub struct Options {
    pub sep: WindowsPathSep,
    pub base_directory: Option<PathBuf>,
    pub canonicalize: bool,
}

pub struct Root {
    pub unc: PathBuf,
    pub mounts: Vec<DrvFsMountPoint>,
}

#[derive(Error, Debug)]
pub enum WslPathError {
    #[error("not absolute path, but didn't choose to canonicalize it")]
    NotAbsolute,
    #[error("failed to canonicalize path")]
    Canonicalization(#[from] io::Error),
}

#[derive(Error, Debug)]
pub enum ConvertOptionsError {
    #[error(transparent)]
    NotWsl(#[from] NotWslError),
    #[error(transparent)]
    WslPath(#[from] WslPathError),
    #[error("error reading mount info from /proc/mounts")]
    Mount(#[from] io::Error),
}

impl Options {
    pub fn init(&mut self) -> Result<(), ConvertOptionsError> {
        if let Some(path) = &self.base_directory {
            if self.canonicalize {
                let path = path.canonicalize()
                    .map_err(WslPathError::Canonicalization)?;
                self.base_directory = Some(path);
            } else if !path.is_absolute() {
                Err(WslPathError::NotAbsolute)?;
            }
        }
        Ok(())
    }
}

impl Root {
    pub fn new(_options: &Options) -> Result<Self, ConvertOptionsError> {
        Ok(Self {
            unc: get_unc_root()
                .map(PathBuf::from)?,
            mounts: get_drvfs_mount_points()
                .map_err(ConvertOptionsError::Mount)?,
        })
    }
}
