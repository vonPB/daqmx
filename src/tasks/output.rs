//! Provides traits around output task behaviours
use crate::daqmx;
use daqmx::bool32;

use crate::daqmx_call;
use crate::types::{DataFillMode, Timeout};
use anyhow::{bail, Result};

#[derive(Clone, Copy, Debug)]
pub struct WriteOptions {
    pub auto_start: bool,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self { auto_start: true }
    }
}

impl WriteOptions {
    /// Controls DAQmx "autoStart" for write calls.
    ///
    /// - `true` (default): the write may start the task immediately.
    /// - `false`: the write only preloads the task's buffer; you must call [`Task::start`]
    ///   (and satisfy any configured start trigger) for the output to begin.
    pub const fn auto_start(mut self, v: bool) -> Self {
        self.auto_start = v;
        self
    }
}

pub trait OutputTask<T>: DAQmxOutput<T> {
    /// Write a single value to the task.
    fn write_scalar(&mut self, value: T, timeout: Timeout) -> Result<()>;

    /// Writes an array of samples to the task where the array can hold multiple channels and/or multiple samples.
    /// See [`OutputTask::write_with_options`] for more details.
    ///
    /// Note: this call automatically sets [`WriteOptions::auto_start`] to true.
    fn write(
        &mut self,
        timeout: Timeout,
        fill_mode: DataFillMode,
        samples_per_channel: Option<u32>,
        buffer: &[T],
    ) -> Result<i32> {
        self.write_with_options(
            timeout,
            fill_mode,
            samples_per_channel,
            buffer,
            WriteOptions::default(),
        )
    }

    /// Writes an array of samples to the task where the array can hold multiple channels and/or multiple samples.
    ///
    /// # Buffer
    /// The buffer should be large enough to contain the number of samples * the number of channels that you want to write.
    ///
    /// # Return
    /// The number of samples written per channel.
    /// Provide [`WriteOptions::auto_start`] to control whether the task starts automatically.
    fn write_with_options(
        &mut self,
        timeout: Timeout,
        fill_mode: DataFillMode,
        samples_per_channel: Option<u32>,
        buffer: &[T],
        opts: WriteOptions,
    ) -> Result<i32> {
        let mut actual_samples_per_channel = 0;
        let requested_samples_per_channel = match samples_per_channel {
            Some(val) => val as i32,
            None => -1,
        };

        if buffer.is_empty() {
            bail!("Buffer is empty, nothing to write.");
        }

        if requested_samples_per_channel != -1
            && buffer.len() % requested_samples_per_channel as usize != 0
        {
            bail!("Buffer length is not a multiple of the requested samples per channel.");
        }

        daqmx_call!(self.daqmx_write(
            requested_samples_per_channel,
            daqmx::bool32::from(opts.auto_start),
            timeout.into(),
            fill_mode.into(),
            buffer.as_ptr(),
            &mut actual_samples_per_channel as *mut i32
        ))?;

        Ok(actual_samples_per_channel)
    }
}

pub trait DAQmxOutput<T> {
    /// Low-level wrapper around the underlying NI-DAQmx write call.
    ///
    /// This exists so implementers only need to provide the final FFI call, while
    /// [`OutputTask::write_with_options`] handles argument validation and common setup.
    ///
    /// # Safety
    /// Implementers must uphold the following:
    ///
    /// - `buffer` must point to at least `N` contiguous elements of `T`, where:
    ///   - If `samples_per_channel == -1`, `N` is the full buffer length chosen by the caller
    ///     (DAQmx interprets this as "all available / full finite buffer", depending on task mode).
    ///   - Otherwise, `N >= samples_per_channel * num_channels` for the configured task.
    /// - `actual_samples_per_channel` must be a valid, writable pointer to an `i32`.
    /// - `self.raw_handle()` (or equivalent) must refer to a valid DAQmx task handle that remains
    ///   valid for the duration of the call.
    /// - The DAQmx function called by the implementation must interpret `T` exactly as the
    ///   element type expected by the DAQmx API for that write (e.g. `f64` for `DAQmxWriteAnalogF64`).
    ///
    /// Violating any of these requirements may cause undefined behavior.
    unsafe fn daqmx_write(
        &mut self,
        samples_per_channel: i32,
        auto_start: bool32,
        timeout: f64,
        fill_mode: bool32,
        buffer: *const T,
        actual_samples_per_channel: *mut i32,
    ) -> i32;
}
