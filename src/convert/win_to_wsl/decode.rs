use optional::Optioned;

use crate::convert::windows_file_name_char::{IllegalWindowsFileNameCharError, WindowsFileNameCharType};

fn check_char(c: u8) -> Result<u8, IllegalWindowsFileNameCharError> {
    use WindowsFileNameCharType::*;
    let char_type: WindowsFileNameCharType = c.into();
    match char_type {
        Legal | Slash => Ok(c),
        _ => Err(c.into()),
    }
}

fn check_path(path: &[u8]) -> Result<(), IllegalWindowsFileNameCharError> {
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
    use WindowsFileNameCharType::*;
    let none = Optioned::none();
    if a != 239 {
        return none;
    }
    match b {
        128 => {
            let d = c - 128;
            match WindowsFileNameCharType::from(d) {
                Low | Reserved => Optioned::some(d),
                _ => none,
            }
        },
        129 => {
            let d = c - 64;
            match WindowsFileNameCharType::from(d) {
                Reserved | BackSlash => Optioned::some(d),
                _ => none,
            }
        },
        _ => none,
    }
}

/// decode a Windows WSL path
/// buf should be pre-reserved by path.len() here
pub fn path(path: &[u8], buf: &mut Vec<u8>) -> Result<(), IllegalWindowsFileNameCharError> {
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
