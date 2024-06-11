//! Based on <https://github.com/WiresmithTech/daqmx-rs>
//!
//!
//! # Examples
//!
//! ```rust
//! use daqmx::channels::*;
//! use daqmx::tasks::*;
//!
//! fn main() -> Result<()> {
//!     let mut task: Task<AnalogInput> = Task::new("scalar")?;
//!     let ch1 = VoltageChannel::new("my name", "PCIe-6363_test/ai0")?.build()?;
//!     task.create_channel(ch1)?;
//!
//!     let res = task.read_scalar(Timeout::Seconds(1.0))?;
//!
//!     Ok(())
//! }
//! ```
//!
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

pub use channels::*;
pub use tasks::*;
pub use types::*;
pub use utils::*;

#[macro_export]
macro_rules! daqmx_call {
    ($l:expr) => {
        $crate::error::handle_error(unsafe { $l })
    };
}
