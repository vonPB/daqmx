use crate::channels::{AnalogChannelBuilderTrait, AnalogChannelTrait, ChannelBuilderInput};
use crate::types::Timeout;
use crate::{daqmx, daqmx_call};
use anyhow::Result;
use std::ptr;

use super::input::{DAQmxInput, InputTask};
use super::{task::AnalogInput, Task};

impl Task<AnalogInput> {
    pub fn create_channel<B: AnalogChannelBuilderTrait + ChannelBuilderInput>(
        &mut self,
        builder: B,
    ) -> Result<()> {
        <B as ChannelBuilderInput>::add_to_task(builder, self.raw_handle())?;
        self.channel_count += 1;
        Ok(())
    }

    pub fn get_channel<C: AnalogChannelTrait<AnalogInput>>(&self, name: &str) -> Result<C> {
        C::new(self.clone(), name)
    }
}

impl InputTask<f64> for Task<AnalogInput> {
    fn read_scalar(&mut self, timeout: Timeout) -> Result<f64> {
        let mut value = 0.0;
        daqmx_call!(daqmx::DAQmxReadAnalogScalarF64(
            self.raw_handle(),
            timeout.into(),
            &mut value,
            ptr::null_mut(),
        ))?;
        Ok(value)
    }
}

impl DAQmxInput<f64> for Task<AnalogInput> {
    unsafe fn daqmx_read(
        &mut self,
        samples_per_channel: i32,
        timeout: f64,
        fill_mode: daqmx::bool32,
        buffer: *mut f64,
        buffer_size: u32,
        actual_samples_per_channel: *mut i32,
    ) -> i32 {
        daqmx::DAQmxReadAnalogF64(
            self.raw_handle(),
            samples_per_channel,
            timeout,
            fill_mode,
            buffer,
            buffer_size,
            actual_samples_per_channel,
            ptr::null_mut(),
        )
    }
}
