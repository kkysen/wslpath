use thiserror::Error;

pub use crate::convert::wsl_to_win::init::Options;

mod init;
mod encode;

#[derive(Error, Debug)]
pub enum ConvertOptionsError {}

#[derive(Error, Debug)]
pub enum ConvertError {}

pub struct Converter {}

impl super::Converter for Converter {
    type Options = Options;
    type OptionsError = ConvertOptionsError;
    type Error = ConvertError;
    
    fn new(_options: Self::Options) -> Result<Self, Self::OptionsError> {
        unimplemented!()
    }
    
    fn convert_into_buf(&self, _path: &mut [u8], _buf: &mut Vec<u8>) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
