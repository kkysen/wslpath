use crate::convert::Slash;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::{io, env};
use same_file::Handle;
use thiserror::Error;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::ffi::OsStrExt;

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

fn get_wsl_distro_name() -> Result<OsString, ConvertOptionsError> {
    env::var_os("WSL_DISTRO_NAME")
        .ok_or(ConvertOptionsError::NotWsl)
}

fn get_unc_root() -> Result<OsString, ConvertOptionsError> {
    let mut distro = get_wsl_distro_name()?.into_vec();
    let prefix = b"//wsl$/";
    let mut path = Vec::with_capacity(prefix.len() + distro.len());
    path.extend_from_slice(prefix);
    path.append(&mut distro);
    let path = OsString::from_vec(path);
    Ok(path)
}

fn get_windows_env_var(var: &str) -> Result<OsString, WindowsEnvVarError> {
    let win_cmd = format!("echo \"%{}%\"", var);
    let output = Command::new("cmd.exe")
        .args(&["/c", win_cmd.as_str()])
        .output()
        .map_err(WindowsEnvVarError::from(var))?;
    let Output {status, stdout, stderr: _ } = output;
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
    let root_handle = Handle::from_path("/").map_err(IoError)?;
    let app_data_local = {
        let username = get_windows_env_var("USERNAME")?;
        let mut path = PathBuf::from("/mnt/c/Users");
        path.push(username);
        path.push("AppData/Local/Packages");
        path
    };
    // rootfs/ under /mnt/c/ has the same inode as / in WSL
    for entry in app_data_local.read_dir().map_err(LocalAppDataNotFound)? {
        let mut path = entry.map_err(IoError)?.path();
        path.push("LocalState/rootfs");
        if let Ok(handle) = Handle::from_path(path.as_path()) {
            if root_handle == handle {
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

struct Root {
    unc: OsString,
    windows_store: Option<PathBuf>,
}

impl Root {
    fn new(options: &Options) -> Result<Self, ConvertOptionsError> {
        Ok(Self {
            unc: get_unc_root()?,
            windows_store: match options.convert_root_loop {
                true => {
                    let path = get_windows_store_root()?
                        .strip_prefix("/mnt/c/")
                        .unwrap() // added /mnt/c/ as a literal, so it must be a prefix
                        .to_path_buf();
                    Some(path)
                },
                false => None,
            },
        })
    }
}

pub struct Converter {
    options: Options,
    root: Root,
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

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("parse error")]
    Parse,
}

impl Converter {
    
    /// convert path sep to to posix path sep
    /// if we're accepting / as the Windows sep, then do nothing
    /// if we're accepting \ as the Windows sep, need to replace them w/ /
    fn convert_sep(&self, path: &mut [u8]) {
        match self.options.sep {
            Slash::Forward => {} // already all /,
            Slash::Backward => {
                for c in path {
                    if *c == Slash::Backward.value() {
                        *c = Slash::Forward.value();
                    }
                }
            },
        }
    }
    
    fn try_fix_root_loop<'a>(&self, path: &'a [u8]) -> Option<&'a [u8]> {
        let root = self
            .root
            .windows_store
            .as_ref()?
            .as_os_str()
            .as_bytes();
        if path.len() < root.len() {
            return None
        }
        if &path[..root.len()] != root {
            return None
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
    fn raw_convert(&self, path: &[u8]) -> Result<Vec<u8>, ConvertError> {
        let verbatim = b"//?/";
        let path = if path.starts_with(verbatim) {
            &path[verbatim.len()..]
        } else {
            path
        };
        let unc_root = self.root.unc.as_bytes();
        let path = match path {
            _ if path.starts_with(unc_root) => {
                path[unc_root.len()..].to_vec()
            }
            [drive, b':', b'/', path @ ..] => {
                let path = self.fix_root_loop(path);
                let len = path.len() - b"C:".len() + b"/mnt/c".len();
                let mut new_path = Vec::with_capacity(len);
                new_path.extend_from_slice(b"/mnt/");
                new_path.push(drive.to_ascii_lowercase());
                new_path.push(b'/');
                new_path.extend_from_slice(path);
                new_path
            },
            _ => return Err(ConvertError::Parse),
        };
        Ok(path)
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
        let mut path = path.into_vec();
        self.convert_sep(path.as_mut_slice());
        let mut path = self.raw_convert(path.as_slice())?;
        self.fix_root_loop(&mut path);
        Ok(path)
            .map(OsString::from_vec)
            .map(PathBuf::from)
    }
}
