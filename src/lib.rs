//! Based on https://github.com/WiresmithTech/daqmx-rs
//! All
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

pub mod daqmx {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod channels;
pub mod error;
pub mod scales;
pub mod tasks;
pub mod types;
pub mod utils;

pub use tasks::Task;
pub use utils::*;

#[macro_export]
macro_rules! daqmx_call {
    ($l:expr) => {
        $crate::error::handle_error(unsafe { $l })
    };
}
