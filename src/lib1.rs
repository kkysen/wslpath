use std::path::Path;
use std::ffi::OsString;
use std::io;
use itertools::Itertools;

fn is_valid_drive(drive: &str) -> bool {
    drive.chars().all(|it| it.is_ascii_alphabetic())
}

fn get_drive(path: &Path) -> Option<OsString> {
    assert!(path.is_absolute());
    if !path.starts_with("/mnt/") {
        return None;
    }
    path.components().nth(2)
        .map(|it| it.as_os_str())
        .and_then(|it| it.to_str())
        .filter(|it| is_valid_drive(it))
        .map(|it| it.to_ascii_uppercase())
        .map(|it| it.into())
}

pub fn is_wsl_windows_abs_path(path: &Path) -> bool {
    get_drive(path).is_some()
}

pub fn is_wsl_windows_path(path: &Path) -> io::Result<bool> {
    match path.is_absolute() {
        true => Ok(is_wsl_windows_abs_path(path)),
        false => std::env::current_dir()
            .map(|it| is_wsl_windows_abs_path(it.as_path())),
    }
}

const SEP: &str = "\\";

fn convert(path: &Path, skip: usize, prefix: OsString) -> OsString {
    path
        .components()
        .skip(skip)
        .map(|it| it.as_os_str())
        .intersperse(SEP.as_ref())
        .fold(prefix, |mut path, part| {
            path.push(part);
            path
        })
}

pub fn wsl_to_windows_abs_path(path: &Path) -> Option<OsString> {
    get_drive(path)
        .map(|mut drive| {
            drive.push(":");
            drive.push(SEP);
            convert(path, 3, drive)
        })
}

pub fn wsl_to_windows_path(path: &Path) -> io::Result<Option<OsString>> {
    Ok(match path.is_absolute() {
        true => wsl_to_windows_abs_path(path),
        false => match is_wsl_windows_path(path)? {
            true => Some(convert(path, 0, "".into())),
            false => None,
        }
    })
    // TODO have to handle path = c when cwd = /mnt
}
