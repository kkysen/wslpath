use std::{env, io};
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

use thiserror::Error;
use std::path::PathBuf;
use proc_mounts::MountIter;
use itertools::Itertools;

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

pub struct DrvFsMountPoint {
    pub wsl: PathBuf,
    pub win: PathBuf,
}

#[derive(Error, Debug)]
#[error("error reading mount info from /proc/mounts")]
pub struct MountError {
    #[from] source: io::Error,
}

pub fn get_drvfs_mount_points() -> Result<Vec<DrvFsMountPoint>, MountError> {
    let mut mounts = Vec::new();
    for mount in MountIter::new()? {
        let mount = mount?;
        if mount.fstype.as_str() == "drvfs" {
            mounts.push(DrvFsMountPoint {
                wsl: mount.dest,
                win: mount.source,
            })
        }
    }
    Ok(mounts)
}
