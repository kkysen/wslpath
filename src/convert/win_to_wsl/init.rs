use std::ffi::OsString;
use std::path::{PathBuf, Path};
use std::{io, env};
use std::process::{Command, Output};
use crate::convert::Slash;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::MetadataExt;
use thiserror::Error;
use crate::convert::win_to_wsl::Converter;
use crate::convert::common::get_unc_root;

#[derive(Error, Debug)]
#[error("Windows environment variable lookup failed for {var}")]
pub struct WindowsEnvVarError {
    var: String,
    source: io::Error,
}

impl WindowsEnvVarError {
    fn from<'a>(var: &'a str) -> impl FnOnce(io::Error) -> Self + 'a {
        move |source| Self {
            var: var.into(),
            source,
        }
    }
}

#[derive(Error, Debug)]
pub enum WindowsStoreRootLookupError {
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    WindowsEnvVar(#[from] WindowsEnvVarError),
    #[error("/mnt/c/\"%USERNAME%/AppData/Local\" not found")]
    LocalAppDataNotFound(io::Error),
    #[error(transparent)]
    IoError(io::Error),
}

#[derive(Error, Debug)]
pub enum ConvertOptionsError {
    #[error("not running on WSL")]
    NotWsl,
    #[error("WSL root not found in Windows Store packages")]
    WindowsStoreRootLookup(#[from] WindowsStoreRootLookupError),
}

fn get_windows_env_var(var: &str) -> Result<OsString, WindowsEnvVarError> {
    let win_cmd = format!("echo %{}%", var);
    let output = Command::new("cmd.exe")
        .args(&["/c", win_cmd.as_str()])
        .output()
        .map_err(WindowsEnvVarError::from(var))?;
    let Output { status, stdout, stderr: _ } = output;
    if !status.success() {
        let code = status.code().unwrap_or_default();
        return Err(code)
            .map_err(io::Error::from_raw_os_error)
            .map_err(WindowsEnvVarError::from(var));
    }
    let mut username = stdout;
    if username.ends_with(&[b'\r', b'\n']) {
        username.truncate(username.len() - 2);
    }
    let username = OsString::from_vec(username);
    Ok(username)
}

fn get_windows_store_root() -> Result<PathBuf, WindowsStoreRootLookupError> {
    use WindowsStoreRootLookupError::{LocalAppDataNotFound, IoError};
    let root_inode = Path::new("/")
        .metadata()
        .map_err(IoError)?
        .ino();
    let app_data_local = {
        let username = get_windows_env_var("USERNAME")?;
        let mut path = PathBuf::from("/mnt/c/Users");
        path.push(username);
        path.push("AppData/Local/Packages");
        path
    };
    // rootfs/ under /mnt/c/ has the same inode as / in WSL, but different device
    for entry in app_data_local.read_dir().map_err(LocalAppDataNotFound)? {
        let mut path = entry.map_err(IoError)?.path();
        path.push("LocalState/rootfs");
        if let Ok(metadata) = path.metadata() {
            if root_inode == metadata.ino() {
                return Ok(path);
            }
        }
    }
    Err(WindowsStoreRootLookupError::NotFound)
}

pub struct Options {
    pub convert_root_loop: bool,
    pub sep: Slash,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            convert_root_loop: true,
            sep: Slash::default(),
        }
    }
}

pub struct Root {
    pub unc: OsString,
    pub windows_store: Option<PathBuf>,
}

impl Root {
    fn new(options: &Options) -> Result<Self, ConvertOptionsError> {
        Ok(Self {
            unc: get_unc_root().ok_or(ConvertOptionsError::NotWsl)?,
            windows_store: match options.convert_root_loop {
                true => {
                    let path = get_windows_store_root()?
                        .strip_prefix("/mnt/c/")
                        .unwrap() // added /mnt/c/ as a literal, so it must be a prefix
                        .to_path_buf();
                    Some(path)
                }
                false => None,
            },
        })
    }
}

impl Converter {
    pub fn new(options: Options) -> Result<Self, ConvertOptionsError> {
        let root = Root::new(&options)?;
        Ok(Self {
            options,
            root,
        })
    }
}
