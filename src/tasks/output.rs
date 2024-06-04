/// Provides traits around output task behaviours - notably reading.
///
/// In future it may expose a writer struct for managing the buffers and providing
/// the different data representations for us.
use crate::daqmx;
use daqmx::bool32;

use crate::daqmx_call;
use crate::types::{DataFillMode, Timeout};
use anyhow::Result;

pub trait OutputTask<T>: DAQmxOutput<T> {
    /// Write a single value to the task.
    fn write_scalar(&mut self, value: T, timeout: Timeout) -> Result<()>;

    /// Writes an array of samples to the task where the array can hold multiple channels and/or multiple samples.
    ///
    /// # Buffer
    ///
    /// The buffer should be large enough to contain the number of samples * the number of channels that you want to write.
    ///
    /// # Return
    /// The number of samples written per channel.
    fn write(
        &mut self,
        timeout: Timeout,
        fill_mode: DataFillMode,
        samples_per_channel: Option<u32>,
        buffer: Vec<T>,
    ) -> Result<i32>
    where
        T: Clone,
    {
        let mut actual_samples_per_channel = 0;
        let requested_samples_per_channel = match samples_per_channel {
            Some(val) => val as i32,
            None => -1,
        };

        daqmx_call!(self.daqmx_write(
            requested_samples_per_channel,
            timeout.into(),
            fill_mode.into(),
            buffer,
            &mut actual_samples_per_channel as *mut i32
        ))?;

        Ok(actual_samples_per_channel)
    }
}

pub trait DAQmxOutput<T> {
    /// A basic wrapper for the daqmx write function so that implementers don't have to repeat common setup for output task.
    unsafe fn daqmx_write(
        &mut self,
        samples_per_channel: i32,
        timeout: f64,
        fill_mode: bool32,
        buffer: Vec<T>,
        actual_samples_per_channel: *mut i32,
    ) -> i32;
}
