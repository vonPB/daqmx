use crate::{
    daqmx::bool32,
    tasks::output::{DAQmxOutput, OutputTask},
    types::Timeout,
};
use anyhow::Result;

use super::SharedMockState;

pub struct MockAnalogOutput {
    state: SharedMockState,
}

impl MockAnalogOutput {
    pub fn new(state: SharedMockState) -> Self {
        Self { state }
    }
}

impl DAQmxOutput<f64> for MockAnalogOutput {
    unsafe fn daqmx_write(
        &mut self,
        samples_per_channel: i32,
        _timeout: f64,
        _fill_mode: bool32,
        buffer: *const f64,
        actual_samples_per_channel: *mut i32,
    ) -> i32 {
        let buffer_slice = std::slice::from_raw_parts(buffer, samples_per_channel as usize);
        let mut values = self.state.analog_values.lock().unwrap();
        values.extend_from_slice(buffer_slice);
        *actual_samples_per_channel = samples_per_channel;
        samples_per_channel
    }
}

impl OutputTask<f64> for MockAnalogOutput {
    fn write_scalar(&mut self, value: f64, _timeout: Timeout) -> Result<()> {
        let mut values = self.state.analog_values.lock().unwrap();
        values.push(value);
        Ok(())
    }
}
