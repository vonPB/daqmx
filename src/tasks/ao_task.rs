use crate::channels::{AnalogChannelBuilderTrait, AnalogChannelTrait};
use crate::types::Timeout;
use crate::{daqmx, daqmx_call};
use anyhow::Result;
use std::ptr;

use super::output::{DAQmxOutput, OutputTask};
use super::{task::AnalogOutput, Task};

impl Task<AnalogOutput> {
    pub fn create_channel<B: AnalogChannelBuilderTrait>(&mut self, builder: B) -> Result<()> {
        builder.add_to_task(self.raw_handle())?;
        self.channel_count += 1;
        Ok(())
    }

    pub fn get_channel<C: AnalogChannelTrait<AnalogOutput>>(&self, name: &str) -> Result<C> {
        C::new(self.clone(), name)
    }
}

impl OutputTask<f64> for Task<AnalogOutput> {
    fn write_scalar(&mut self, value: f64, timeout: Timeout) -> Result<()> {
        daqmx_call!(daqmx::DAQmxWriteAnalogScalarF64(
            self.raw_handle(),
            1,
            timeout.into(),
            value,
            ptr::null_mut()
        ))?;
        Ok(())
    }
}

impl DAQmxOutput<f64> for Task<AnalogOutput> {
    unsafe fn daqmx_write(
        &mut self,
        samples_per_channel: i32,
        timeout: f64,
        fill_mode: daqmx::bool32,
        buffer: *const f64,
        actual_samples_per_channel: *mut i32,
    ) -> i32 {
        let autostart = daqmx::bool32::from(true);

        daqmx::DAQmxWriteAnalogF64(
            self.raw_handle(),
            samples_per_channel,
            autostart,
            timeout,
            fill_mode,
            buffer,
            actual_samples_per_channel,
            ptr::null_mut(),
        )
    }
}
