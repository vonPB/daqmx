mod analog_channels;
mod digital_channels;

pub use analog_channels::*;
pub use digital_channels::*;

use crate::daqmx::TaskHandle;
use crate::error::{handle_error, string_property_size_error, DaqmxError};
use crate::{daqmx, daqmx_call};
use anyhow::Result;
use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

trait Channel {
    fn raw_handle(&self) -> *mut c_void;
    fn name(&self) -> &CStr;

    ///Read a channel property as a string, given a raw DAQmx Function.
    fn read_channel_property_string(
        &self,
        daqmx_fn: unsafe extern "C" fn(daqmx::TaskHandle, *const c_char, *mut c_char, u32) -> i32,
    ) -> Result<String> {
        let return_value = unsafe {
            daqmx_fn(
                self.raw_handle(),
                self.name().as_ptr(),
                std::ptr::null_mut(),
                0,
            )
        };

        if return_value < 0 {
            handle_error(return_value)?;
        }

        let buffer_size = return_value as u32;

        let mut buffer = vec![0u8; return_value as usize];

        let return_value = unsafe {
            daqmx_fn(
                self.raw_handle(),
                self.name().as_ptr(),
                buffer.as_mut_ptr() as *mut c_char,
                buffer_size,
            )
        };

        let should_retry = string_property_size_error(return_value)?;

        if should_retry {
            // Just error for now - will review retries in the future.
            return Err(DaqmxError::StringPropertyLengthChanged.into());
        }

        //pop the null off.
        buffer.pop();
        Ok(String::from_utf8(buffer)?)
    }

    fn read_channel_property<T: Default + Copy>(
        &self,
        daqmx_fn: unsafe extern "C" fn(daqmx::TaskHandle, *const c_char, *mut T) -> i32,
    ) -> Result<T> {
        let mut value: T = T::default();

        daqmx_call!(daqmx_fn(
            self.raw_handle(),
            self.name().as_ptr(),
            &mut value
        ))?;

        Ok(value)
    }
}

pub trait ChannelBuilderInput {
    fn add_to_task(self, task: TaskHandle) -> Result<()>;
}

pub trait ChannelBuilderOutput {
    fn add_to_task(self, task: TaskHandle) -> Result<()>;
}
