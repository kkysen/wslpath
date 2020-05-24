use std::env;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

pub fn get_wsl_distro_name() -> Option<OsString> {
    env::var_os("WSL_DISTRO_NAME")
}

pub fn get_unc_root() -> Option<OsString> {
    let mut distro = get_wsl_distro_name()?.into_vec();
    let prefix = b"//wsl$/";
    let mut path = Vec::with_capacity(prefix.len() + distro.len());
    path.extend_from_slice(prefix);
    path.append(&mut distro);
    let path = OsString::from_vec(path);
    Some(path)
}
