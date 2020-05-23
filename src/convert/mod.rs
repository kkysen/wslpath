pub mod win_to_wsl;
pub mod wsl_to_win;
mod common;

use thiserror::Error;

#[derive(Clone, Copy)]
pub enum Slash {
    Forward,
    Backward,
}

impl Default for Slash {
    fn default() -> Self {
        Slash::Backward
    }
}

impl Slash {
    pub fn value(&self) -> u8 {
        use Slash::*;
        match self {
            Forward => b'/',
            Backward => b'\\',
        }
    }
}

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

impl From<&Slash> for WindowsFileNameCharType {
    fn from(slash: &Slash) -> Self {
        use Slash::*;
        match slash {
            Forward => WindowsFileNameCharType::Slash,
            Backward => WindowsFileNameCharType::BackSlash,
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
