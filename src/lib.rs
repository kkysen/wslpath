#![feature(option_result_contains)]

#[cfg(not(unix))]
compile_error!("This crate only works on WSL.");

pub mod convert;
pub mod util;
