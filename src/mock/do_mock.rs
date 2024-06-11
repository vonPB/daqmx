use crate::{
    daqmx::bool32,
    tasks::output::{DAQmxOutput, OutputTask},
    types::Timeout,
};

use super::SharedMockState;

use anyhow::Result;

pub struct MockDigitalOutput {
    state: SharedMockState,
}

impl MockDigitalOutput {
    pub fn new(state: SharedMockState) -> Self {
        Self { state }
    }
}

impl DAQmxOutput<u32> for MockDigitalOutput {
    unsafe fn daqmx_write(
        &mut self,
        samples_per_channel: i32,
        _timeout: f64,
        _fill_mode: bool32,
        buffer: *const u32,
        actual_samples_per_channel: *mut i32,
    ) -> i32 {
        let buffer_slice = std::slice::from_raw_parts(buffer, samples_per_channel as usize);
        let mut values = self.state.digital_values.lock().unwrap();
        values.extend_from_slice(buffer_slice);
        *actual_samples_per_channel = samples_per_channel;
        samples_per_channel
    }
}

impl OutputTask<u32> for MockDigitalOutput {
    fn write_scalar(&mut self, value: u32, _timeout: Timeout) -> Result<()> {
        let mut values = self.state.digital_values.lock().unwrap();
        values.push(value);
        Ok(())
    }
}
