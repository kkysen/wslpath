use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::fmt;
use crate::convert::{InputPathSeparator, OutputPathSeparator};

#[derive(Debug)]
pub enum LineSep {
    Null,
    LF,
    CRLF,
}

impl Default for LineSep {
    fn default() -> Self {
        use LineSep::*;
        LF
    }
}

impl Display for LineSep {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use LineSep::*;
        let s = match self {
            Null => "null",
            LF => "LF",
            CRLF => "CRLF",
        };
        write!(f, "{}", s);
        Ok(())
    }
}

impl TryFrom<&str> for LineSep {
    type Error = ();
    
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let s = s.to_ascii_uppercase();
        use LineSep::*;
        let this = match s.as_str() {
            "NULL" => Null,
            "LF" => LF,
            "CRLF" => CRLF,
            _ => return Err(()),
        };
        Ok(this)
    }
}

struct OneCharSep(u8);

impl InputPathSeparator for OneCharSep {
    fn matches(&self, c: u8) -> bool {
        c == self.0
    }
}

impl OutputPathSeparator for OneCharSep {
    fn write_to_buf(&self, buf: &mut Vec<u8>) {
        buf.push(self.0);
    }
}

struct TwoCharSep(u8, u8);

impl InputPathSeparator for TwoCharSep {
    fn matches(&self, c: u8) -> bool {
        c == self.0 || c == self.1
    }
}

impl OutputPathSeparator for TwoCharSep {
    fn write_to_buf(&self, buf: &mut Vec<u8>) {
        buf.push(self.0);
        buf.push(self.1);
    }
}

impl LineSep {
    pub fn as_str(&self) -> &'static str {
        use LineSep::*;
        match self {
            Null => "\0",
            LF => "\n",
            CRLF => "\r\n",
        }
    }
    
    pub fn sep(&self) -> impl InputPathSeparator + OutputPathSeparator {
        use LineSep::*;
        match self {
            Null => OneCharSep(0),
            LF => OneCharSep(b'\n'),
            CRLF => TwoCharSep(b'\r', b'\n'),
        }
    }
}
