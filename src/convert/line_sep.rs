use crate::convert::{InputPathSeparator, OutputPathSeparator};
use crate::util::enum_arg::EnumArg;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, Debug)]
pub enum LineSep {
    Null,
    LF,
    CRLF,
}

impl LineSep {
    pub fn value(&self) -> &'static str {
        use LineSep::*;
        match self {
            Null => "\0",
            LF => "\n",
            CRLF => "\r\n",
        }
    }
}

impl EnumArg for LineSep {
    fn variants() -> &'static [Self] {
        use LineSep::*;
        &[Null, LF, CRLF]
    }
    
    fn displays(&self) -> &'static [&'static str] {
        use LineSep::*;
        match self {
            Null => &["null", "0", r"\0"],
            LF => &["LF", r"\n"],
            CRLF => &["CRLF", r"\r\n"],
        }
    }
}

impl Display for LineSep {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        EnumArg::fmt(self, f)
    }
}

impl FromStr for LineSep {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EnumArg::from_str(s)
    }
}

impl Default for LineSep {
    fn default() -> Self {
        use LineSep::*;
        LF
    }
}

// ideal for const-generics when they're implemented

impl InputPathSeparator for LineSep {
    fn matches(&self, c: u8) -> bool {
        use LineSep::*;
        match self {
            Null => c == 0,
            LF => c == b'\n',
            CRLF => c == b'\r' || c == b'\n',
        }
    }
}

impl OutputPathSeparator for LineSep {
    fn write_to_buf(&self, buf: &mut Vec<u8>) {
        use LineSep::*;
        match self {
            Null => buf.push(0),
            LF => buf.push(b'\n'),
            CRLF => {
                buf.push(b'\r');
                buf.push(b'\n');
            }
        }
    }
}
