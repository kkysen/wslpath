mod init;
mod encode;

use thiserror::Error;
pub use crate::convert::wsl_to_win::init::Options;

#[derive(Error, Debug)]
pub enum ConvertOptionsError {}

#[derive(Error, Debug)]
pub enum ConvertError {}

pub struct Converter {}

impl super::Converter for Converter {
    type Options = Options;
    type OptionsError = ConvertOptionsError;
    type Error = ConvertError;
    
    fn new(options: Self::Options) -> Result<Self, Self::OptionsError> {
        unimplemented!()
    }
    
    fn convert_into_buf(&self, path: &mut [u8], buf: &mut Vec<u8>) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
