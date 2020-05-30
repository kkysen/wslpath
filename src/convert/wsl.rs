use std::env;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

use thiserror::Error;

#[derive(Error, Debug)]
#[error("not running on WSL")]
pub struct NotWslError {}

pub fn get_wsl_distro_name() -> Result<OsString, NotWslError> {
    env::var_os("WSL_DISTRO_NAME")
        .ok_or(NotWslError {})
}

pub fn get_unc_root() -> Result<OsString, NotWslError> {
    let mut distro = get_wsl_distro_name()?.into_vec();
    let prefix = b"//wsl$/";
    let mut path = Vec::with_capacity(prefix.len() + distro.len());
    path.extend_from_slice(prefix);
    path.append(&mut distro);
    let path = OsString::from_vec(path);
    Ok(path)
}
