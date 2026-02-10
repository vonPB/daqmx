/// Provides traits around input task behaviours - notably reading.
///
/// In future it may expose a reader struct for managing the buffers and providing
/// the different data representations for us.
use crate::daqmx;
use daqmx::bool32;

use crate::daqmx_call;
use crate::types::{DataFillMode, Timeout};
use anyhow::{bail, Result};

pub trait InputTask<T>: DAQmxInput<T> {
    /// Read a single value from the task with the given timeout.
    fn read_scalar(&mut self, timeout: Timeout) -> Result<T>;

    /// Reads an array of samples from the task where the array can hold multiple channels and/or multiple samples.
    ///
    /// # Samples Per Channel Behaviour
    ///
    /// * Writing [`None`] on a finite task will wait until the full acquisition is ready to read.
    /// * Writing [`None`] on a continuous task will read all of the samples available in the buffer.
    /// * If you attempt to read more samples than can fit into the buffer, then only the samples that fit in the buffer will be read.
    ///
    /// # Buffer
    ///
    /// The buffer should be large enough to contain the number of samples * the number of channels that you want to read.
    ///
    /// # Return
    /// The number of samples read per channel.
    ///
    /// TODO: If we timeout we may still read samples - that needs to be expressed with a more complicated return type.
    fn read(
        &mut self,
        timeout: Timeout,
        fill_mode: DataFillMode,
        samples_per_channel: Option<u32>,
        buffer: &mut [T],
    ) -> Result<i32> {
        let mut actual_samples_per_channel = 0;
        let requested_samples_per_channel = match samples_per_channel {
            Some(val) => val as i32,
            None => -1,
        };

        if buffer.is_empty() {
            bail!("Read buffer is empty, nothing to read into.");
        }

        // Just saturate the buffer size at u32 boundary.
        // If it is larger, this will still be memory safe.
        let buffer_length = buffer.len().try_into().unwrap_or(u32::MAX);

        daqmx_call!(self.daqmx_read(
            requested_samples_per_channel,
            timeout.into(),
            fill_mode.into(),
            buffer,
            buffer_length,
            &mut actual_samples_per_channel as *mut i32
        ))?;

        Ok(actual_samples_per_channel)
    }
}

pub trait DAQmxInput<T> {
    /// Low-level wrapper around the underlying NI-DAQmx read call.
    ///
    /// This exists so implementers only need to provide the final FFI call, while
    /// [`InputTask::read`] handles argument normalization and common setup.
    ///
    /// # Safety
    /// Implementers must uphold the following:
    ///
    /// - `buffer` must be valid for writes for the duration of the call.
    /// - `buffer_size` must be the number of elements available in `buffer` that the underlying
    ///   DAQmx function is allowed to write into (not bytes). Typically this should be
    ///   `buffer.len()` clamped to the DAQmx API's supported maximum.
    /// - The implementation must not write more than `buffer_size` elements into `buffer`,
    ///   and must not write past `buffer.len()` regardless of `buffer_size`.
    /// - `actual_samples_per_channel` must be a valid, writable pointer to an `i32`.
    /// - `self.raw_handle()` (or equivalent) must refer to a valid DAQmx task handle that remains
    ///   valid for the duration of the call.
    /// - The DAQmx function called by the implementation must interpret `T` exactly as the
    ///   element type expected by the DAQmx API for that read (e.g. `f64` for `DAQmxReadAnalogF64`,
    ///   `u8`/`i16` for certain digital reads, etc.).
    ///
    /// Violating any of these requirements may cause undefined behavior.
    unsafe fn daqmx_read(
        &mut self,
        samples_per_channel: i32,
        timeout: f64,
        fill_mode: bool32,
        buffer: &mut [T],
        buffer_size: u32,
        actual_samples_per_channel: *mut i32,
    ) -> i32;
}
