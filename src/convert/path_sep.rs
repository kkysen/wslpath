use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub enum WindowsPathSep {
    Slash,
    BackSlash,
}

impl Default for WindowsPathSep {
    fn default() -> Self {
        use WindowsPathSep::*;
        BackSlash
    }
}

impl Into<u8> for WindowsPathSep {
    fn into(self) -> u8 {
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

impl TryFrom<&str> for WindowsPathSep {
    type Error = ();
    
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        use WindowsPathSep::*;
        let this = match s {
            r"/" => Slash,
            r"\" => BackSlash,
            _ => return Err(()),
        };
        Ok(this)
    }
}

impl Display for WindowsPathSep {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.into() as u8);
        Ok(())
    }
}
