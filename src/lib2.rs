use std::ffi::{OsStr, OsString};

use crate::PathType::WSL;

pub enum Error {
    // TODO
}

pub type Result<T> = std::Result<T, Error>;

#[derive(Debug, Copy, PartialEq, Eq)]
pub enum PathType {
    WSL,
    Windows,
    WindowsForwardSlash,
}

impl PathType {
    pub fn is_wsl(&self) -> bool {
        self == WSL
    }
    
    pub fn is_windows(&self) -> bool {
        !self.is_wsl()
    }
    
    pub fn detect(path: &OsStr) -> Result<PathType> {
        todo!()
    }
}

pub fn convert(path: &OsStr, to_path_type: PathType, absolute: bool) -> Result<OsString> {
    // if WSL => WSL
    //     if not absolute or already absolute
    //         no conversion, just to_owned()
    //     else
    //         if os is unix
    //             use std::fs
    //         else
    //             convert
    std::fs::canonicalize()
    let from_path_type: PathType = PathType::detect(path)?;
    if from_path_type.is_wsl() == to_path_type.is_wsl() {
        return Ok(path.to_owned());
    }
    let convert = match from_path_type.is_wsl() {
        true => wsl_to_windows_path,
        false => windows_to_wsl_path,
    };
    convert(path)
}

fn wsl_to_windows_path(path: &OsStr) -> Result<OsString> {
    todo!()
}

fn windows_to_wsl_path(path: &OsStr) -> Result<OsString> {
    todo!()
}
