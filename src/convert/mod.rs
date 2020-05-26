use std::{fmt, io, iter};
use std::cmp::{max, min};
use std::ffi::{OsStr, OsString};
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::Read;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use thiserror::Error;

pub mod win_to_wsl;
pub mod wsl_to_win;
mod wsl;
mod windows_file_name_char;
pub mod path_sep;
pub mod line_sep;

pub trait InputPathSeparator {
    fn matches(&self, c: u8) -> bool;
}

impl<T: InputPathSeparator + ?Sized> InputPathSeparator for &T {
    fn matches(&self, c: u8) -> bool {
        T::matches(self, c)
    }
}

pub trait OutputPathSeparator {
    fn write_to_buf(&self, buf: &mut Vec<u8>);
}

impl<T: OutputPathSeparator + ?Sized> OutputPathSeparator for &T {
    fn write_to_buf(&self, buf: &mut Vec<u8>) {
        T::write_to_buf(self, buf)
    }
}

pub struct PathSeparators<InputSep, OutputSep>
    where InputSep: InputPathSeparator,
          OutputSep: OutputPathSeparator {
    pub input: InputSep,
    pub output: OutputSep,
}

// compiler bug prevents me from doing this
// derive(Debug) requires that all generic parameters impl Debug,
// even though only C::Error is used in the fields
// #[derive(Error, Debug)]
// pub struct OneConvertError<'a, C: Converter> {
//     index: usize,
//     path: &'a [u8],
//     source: C::Error,
// }

#[derive(Error, Debug)]
pub struct OneConvertError<E: std::error::Error + 'static> {
    index: usize,
    path: Vec<u8>,
    source: E,
}

impl<E: std::error::Error + 'static> Display for OneConvertError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "error converting {:#?} at index {}: {:#?}",
               OsStr::from_bytes(self.path.as_slice()),
               self.index,
               self.source,
        )?;
        Ok(())
    }
}

pub struct BulkConversion<C: Converter> {
    pub paths: OsString,
    pub remainder_index: usize,
    pub errors: Vec<OneConvertError<C::Error>>,
}

// I would #[derive(Default)] this,
// but it'd require C: Default b/c of the derive bug
impl<C: Converter> Default for BulkConversion<C> {
    fn default() -> Self {
        Self {
            paths: OsString::default(),
            remainder_index: usize::default(),
            errors: Vec::default(),
        }
    }
}

pub trait Converter where Self: Sized {
    type Options;
    type OptionsError: std::error::Error + Send + Sync + 'static;
    type Error: std::error::Error + Send + Sync + 'static;
    
    fn new(options: Self::Options) -> Result<Self, Self::OptionsError>;
    
    /// Lower-level version of [`convert`].
    /// Takes a [`&mut [u8]`] directly to avoid any copies and allow slices.
    /// Outputs into a [`Vec<u8>`] to avoid extra copies on return.
    /// This is useful for minimizing overhead on bulk conversions.
    fn convert_into_buf(&self, path: &mut [u8], buf: &mut Vec<u8>) -> Result<(), Self::Error>;
    
    /// Convert an absolute path.
    fn convert<S: AsRef<OsStr> + ?Sized>(&self, path: &S) -> Result<OsString, Self::Error> {
        let mut path = path
            .as_ref()
            .to_os_string()
            .into_vec();
        let mut buf = Vec::new();
        self.convert_into_buf(path.as_mut_slice(), &mut buf)?;
        Ok(buf)
            .map(OsString::from_vec)
    }
    
    /// Convert all of [`paths`], interpreted as multiple paths ended by [`input_sep`].
    /// If the end of [`paths`] does not end in an [`input_sep`],
    /// then that part is returned in the remainder field.
    /// The output paths are separated by [`output_sep`], including a trailing separator.
    fn convert_all<'a, InputSep, OutputSep>(
        &self, paths: &'a mut [u8],
        seps: &PathSeparators<InputSep, OutputSep>,
    ) -> BulkConversion<Self>
        where InputSep: InputPathSeparator,
              OutputSep: OutputPathSeparator {
        let empty = BulkConversion::default();
        // if paths is empty, do nothing
        let last_byte = match paths.last() {
            // empty slice
            None => return empty,
            Some(c) => *c,
        };
        // if paths doesn't end in an input_sep,
        // truncate paths from the end until it does end in an input_sep
        // also store the remainder to return
        let remainder_index = if !seps.input.matches(last_byte) {
            let i = match paths.iter()
                .rposition(|c| seps.input.matches(*c)) {
                None => return empty,
                Some(i) => i,
            };
            i + 1
        } else {
            paths.len()
        };
        let paths = &mut paths[..remainder_index];
        
        // at most one more allocation
        let mut buf = Vec::with_capacity(paths.len());
        let mut errors = Vec::new();
        for (index, path) in paths
            .split_mut(|c| seps.input.matches(*c))
            .filter(|it| !it.is_empty())
            .enumerate() {
            // if index % 100000 == 0 {
            //     dbg!(index);
            // }
            let result = self.convert_into_buf(path, &mut buf);
            if let Err(source) = result {
                errors.push(OneConvertError {
                    index,
                    path: path.into(),
                    source,
                });
            }
            seps.output.write_to_buf(&mut buf);
        }
        let paths = OsString::from_vec(buf);
        BulkConversion {
            paths,
            remainder_index,
            errors,
        }
    }
    
    fn convert_file<'a, P, InputSep, OutputSep>(
        &'a self,
        path: P,
        seps: &'a PathSeparators<InputSep, OutputSep>,
        buffer_size_blocks: BufferSizeBlocks,
    ) -> Result<ConversionIterator<'a, Self, InputSep, OutputSep>, ConvertFileError>
        where P: AsRef<Path>,
              InputSep: InputPathSeparator,
              OutputSep: OutputPathSeparator {
        ConversionIterator::new(self, path.as_ref(), seps, buffer_size_blocks)
    }
}

#[derive(Error, Debug)]
pub enum ConvertFileError {
    #[error(transparent)]
    IOError(#[from] io::Error),
    #[error("is a directory")]
    IsADirectory,
}

pub struct BufferSizeBlocks {
    pub min: u64,
    pub max: u64,
}

impl Default for BufferSizeBlocks {
    fn default() -> Self {
        Self {
            min: 16,
            max: u64::MAX,
        }
    }
}

pub struct ConversionIterator<'a, C, InputSep, OutputSep>
    where C: Converter,
          InputSep: InputPathSeparator,
          OutputSep: OutputPathSeparator {
    converter: &'a C,
    separators: &'a PathSeparators<InputSep, OutputSep>,
    file: File,
    file_buf_len: usize,
    buf: Vec<u8>,
}

impl<'a, C, InputSep, OutputSep> ConversionIterator<'a, C, InputSep, OutputSep>
    where C: Converter,
          InputSep: InputPathSeparator,
          OutputSep: OutputPathSeparator {
    fn new(converter: &'a C,
           path: &Path,
           separators: &'a PathSeparators<InputSep, OutputSep>,
           buffer_size_blocks: BufferSizeBlocks,
    ) -> Result<Self, ConvertFileError> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_type = metadata.file_type();
        if file_type.is_dir() {
            return Err(ConvertFileError::IsADirectory);
        }
        let block_size = metadata.blksize();
        let file_buf_len = if file_type.is_file() || file_type.is_symlink() {
            min(metadata.len(), buffer_size_blocks.max.saturating_mul(block_size))
        } else {
            max(1, buffer_size_blocks.min) * block_size
        } as usize;
        let this = Self {
            converter,
            separators,
            file,
            file_buf_len,
            buf: Vec::new(),
        };
        Ok(this)
    }
    
    pub fn buf_len(&self) -> usize {
        self.file_buf_len
    }
    
    // fn read(&mut self) -> io::Result<&mut [u8]> {
    //
    // }
}

impl<'a, C, InputSep, OutputSep> Iterator for ConversionIterator<'a, C, InputSep, OutputSep>
    where C: Converter,
          InputSep: InputPathSeparator,
          OutputSep: OutputPathSeparator {
    type Item = io::Result<BulkConversion<C>>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let start = self.buf.len();
        let len = self.file_buf_len;
        self.buf.reserve(len);
        unsafe {
            // I could zero out this uninitialized memory instead,
            // but that'd just be a waste
            // since the kernel will fill it when I read the file
            // I just need to make sure I don't read beyond
            self.buf.set_len(start + len);
        }
        let read_buf = &mut self.buf[start..start + len];
        match self.file.read(read_buf) {
            Err(e) => return Some(Err({
                // assume no bytes were read
                self.buf.truncate(len);
                e
            })),
            Ok(bytes_read) => {
                // only return bytes actually read into buf
                // don't return any uninitialized memory
                self.buf.truncate(start + bytes_read);
                if bytes_read == 0 {
                    return None;
                }
                let paths = self.buf.as_mut_slice();
                let converted = self.converter.convert_all(paths, self.separators);
                // remove everything except remainder
                self.buf.splice(..converted.remainder_index, iter::empty::<u8>());
                let converted = BulkConversion {
                    remainder_index: 0,
                    ..converted
                };
                Some(Ok(converted))
            }
        }
    }
}
