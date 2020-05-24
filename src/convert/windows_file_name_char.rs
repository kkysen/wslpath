use crate::convert::path_sep::WindowsPathSep;

#[derive(Debug, Eq, PartialEq)]
pub enum WindowsFileNameCharType {
    Null,
    Low,
    Reserved,
    Slash,
    BackSlash,
    Legal,
}

impl From<u8> for WindowsFileNameCharType {
    fn from(c: u8) -> Self {
        use WindowsFileNameCharType::*;
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

impl From<&WindowsPathSep> for WindowsFileNameCharType {
    fn from(slash: &WindowsPathSep) -> Self {
        use WindowsPathSep::*;
        match slash {
            Slash => WindowsFileNameCharType::Slash,
            BackSlash => WindowsFileNameCharType::BackSlash,
        }
    }
}

#[derive(Error, Debug)]
#[error("illegal windows filename char: {char:?} of type {char_type:?}")]
pub struct IllegalWindowsFileNameCharError {
    char: char,
    char_type: WindowsFileNameCharType,
}

impl From<u8> for IllegalWindowsFileNameCharError {
    fn from(c: u8) -> Self {
        Self {
            char: c as char,
            char_type: c.into(),
        }
    }
}
