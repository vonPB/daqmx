use crate::{
    daqmx::bool32,
    tasks::{DAQmxInput, InputTask},
    types::Timeout,
};
use anyhow::Result;

use super::SharedMockState;

pub struct MockAnalogInput {
    state: SharedMockState,
    read_index: usize,
}

impl MockAnalogInput {
    pub fn new(state: SharedMockState) -> Self {
        Self {
            state,
            read_index: 0,
        }
    }
}

impl DAQmxInput<f64> for MockAnalogInput {
    unsafe fn daqmx_read(
        &mut self,
        samples_per_channel: i32,
        _timeout: f64,
        _fill_mode: bool32,
        buffer: &mut [f64],
        buffer_size: u32,
        actual_samples_per_channel: *mut i32,
    ) -> i32 {
        let mut values = self.state.analog_values.lock().unwrap();
        let samples_to_read = samples_per_channel.min(buffer_size as i32) as usize;
        let samples_to_read = samples_to_read.min(values.len() - self.read_index);

        for i in 0..samples_to_read {
            buffer[i] = values[self.read_index + i];
        }
        self.read_index += samples_to_read;

        *actual_samples_per_channel = samples_to_read as i32;
        samples_to_read as i32
    }
}

impl InputTask<f64> for MockAnalogInput {
    fn read_scalar(&mut self, _timeout: Timeout) -> Result<f64> {
        let mut values = self.state.analog_values.lock().unwrap();
        if self.read_index < values.len() {
            let value = values[self.read_index];
            self.read_index += 1;
            Ok(value)
        } else {
            Err(anyhow::anyhow!("No more data to read"))
        }
    }
}
