use std::convert::{TryFrom, TryInto};
use crate::util::enum_arg::EnumArg;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
pub enum WindowsPathSep {
    Slash,
    BackSlash,
}

impl WindowsPathSep {
    pub fn value(&self) -> u8 {
        use WindowsPathSep::*;
        match self {
            Slash => b'/',
            BackSlash => b'\\',
        }
    }
}

impl TryFrom<char> for WindowsPathSep {
    type Error = ();
    
    fn try_from(c: char) -> Result<Self, Self::Error> {
        use WindowsPathSep::*;
        let this = match c {
            '/' => Slash,
            '\\' => BackSlash,
            _ => return Err(()),
        };
        Ok(this)
    }
}

impl TryFrom<u8> for WindowsPathSep {
    type Error = ();
    
    fn try_from(c: u8) -> Result<Self, Self::Error> {
        (c as char).try_into()
    }
}

impl EnumArg for WindowsPathSep {
    fn variants() -> &'static [Self] {
        use WindowsPathSep::*;
        &[Slash, BackSlash]
    }
    
    fn displays(&self) -> &'static [&'static str] {
        use WindowsPathSep::*;
        match self {
            Slash => &["/"],
            BackSlash => &[r"\"],
        }
    }
}

impl Display for WindowsPathSep {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        EnumArg::fmt(self, f)
    }
}

impl FromStr for WindowsPathSep {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EnumArg::from_str(s)
    }
}

impl Default for WindowsPathSep {
    fn default() -> Self {
        use WindowsPathSep::*;
        BackSlash
    }
}
