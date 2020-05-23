use crate::convert::Slash;
use std::ffi::{OsString, OsStr};
use std::path::{PathBuf, Path};
use std::process::{Command, Output};
use std::{io, env};
use thiserror::Error;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use crate::convert::win_to_wsl::decode::IllegalFileNameCharError;

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
    let win_cmd = format!("echo %{}%", var);
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
            },
        }
    }
}

mod decode {
    use optional::Optioned;
    use crate::convert::Slash;
    use thiserror::Error;
    
    #[derive(Debug, Eq, PartialEq)]
    pub enum FileNameCharType {
        Null,
        Low,
        Reserved,
        Slash,
        BackSlash,
        Legal,
    }
    
    impl From<u8> for FileNameCharType {
        fn from(c: u8) -> Self {
            use FileNameCharType::*;
            match c {
                0 => Null,
                _ if c > 0 && c < b' ' => Low,
                b'/' => Slash,
                b'\\' => BackSlash,
                b'"' | b'*' | b':' | b'<' | b'>' | b'?' | b'|' => Reserved,
                _ => Legal,
            }
        }
    }
    
    impl From<&Slash> for FileNameCharType {
        fn from(slash: &Slash) -> Self {
            use Slash::*;
            match slash {
                Forward => FileNameCharType::Slash,
                Backward => FileNameCharType::BackSlash,
            }
        }
    }
    
    #[derive(Error, Debug)]
    #[error("illegal windows filename char: {char:?} of type {char_type:?}")]
    pub struct IllegalFileNameCharError {
        char: char,
        char_type: FileNameCharType,
    }
    
    fn check_char(c: u8) -> Result<u8, IllegalFileNameCharError> {
        use FileNameCharType::*;
        let char_type: FileNameCharType = c.into();
        match char_type {
            Legal | Slash => Ok(c),
            _ => Err(IllegalFileNameCharError {
                char: c as char,
                char_type,
            }),
        }
    }
    
    fn check_path(path: &[u8]) -> Result<(), IllegalFileNameCharError> {
        for c in path {
            check_char(*c)?;
        }
        Ok(())
    }
    
    /// decode a Windows WSL path codepoint
    /// illegal windows filename chars are encoded as UTF-8
    /// see `encoding.py`
    /// illegal chars `c` are encoded as `'\f000' + c`
    /// `'\f000'` is 3-bytes, so (a, b, c)
    /// TODO check assembly for Optioned::<u8>
    pub fn codepoint(codepoint: [u8; 3]) -> Optioned::<u8> {
        let [a, b, c] = codepoint;
        use FileNameCharType::*;
        let none = Optioned::none();
        if a != 239 {
            return none;
        }
        match b {
            128 => {
                let d = c - 128;
                match FileNameCharType::from(d) {
                    Low | Reserved => Optioned::some(d),
                    _ => none,
                }
            },
            129 => {
                let d = c - 64;
                match FileNameCharType::from(d) {
                    Reserved | BackSlash => Optioned::some(d),
                    _ => none,
                }
            },
            _ => none,
        }
    }
    
    /// decode a Windows WSL path
    /// buf should be pre-reserved by path.len() here
    pub fn path(path: &[u8], buf: &mut Vec<u8>) -> Result<(), IllegalFileNameCharError> {
        check_path(path)?;
        // we're decoding multi-byte codepoints to bytes, so this is an overestimate
        buf.reserve(path.len());
        let mut i = 0;
        while i + 2 < path.len() {
            // '\f000' is 3-bytes
            let a = path[i + 0];
            let b = path[i + 1];
            let c = path[i + 2];
            let d= match codepoint([a, b, c]).into_option() {
                Some(d) => {
                    i += 3;
                    d
                },
                None => {
                    i += 1;
                    a
                },
            };
            buf.push(d);
        }
        buf.extend_from_slice(&path[i..]);
        Ok(())
    }
    
}

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("parse error")]
    Parse,
    #[error(transparent)]
    IllegalFileNameChar(#[from] IllegalFileNameCharError),
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
            },
            _ => return Err(ConvertError::Parse),
        };
        Ok(())
    }
    
    pub fn convert_into_buf(&self, path: OsString, buf: &mut Vec<u8>) -> Result<(), ConvertError> {
        let mut path = path.into_vec();
        // dbg!(&OsStr::from_bytes(path.as_slice()));
        self.options.sep.convert_sep(path.as_mut_slice());
        // dbg!(&OsStr::from_bytes(path.as_slice()));
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
