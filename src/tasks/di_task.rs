use crate::channels::{ChannelBuilderInput, DigitalChannelTrait};
use crate::types::Timeout;
use crate::{daqmx, daqmx_call};
use anyhow::Result;
use std::ptr;

use super::input::{DAQmxInput, InputTask};
use super::{task::DigitalInput, Task};

impl Task<DigitalInput> {
    pub fn create_channel<B: ChannelBuilderInput>(&mut self, builder: B) -> Result<()> {
        builder.add_to_task(self.raw_handle())?;
        self.channel_count += 1;
        Ok(())
    }

    pub fn get_channel<C: DigitalChannelTrait<DigitalInput>>(&self, name: &str) -> Result<C> {
        C::new(self.clone(), name)
    }
}

impl InputTask<u8> for Task<DigitalInput> {
    fn read_scalar(&mut self, timeout: Timeout) -> Result<u8> {
        let mut value = 0;
        daqmx_call!(daqmx::DAQmxReadDigitalScalarU32(
            self.raw_handle(),
            timeout.into(),
            &mut value,
            ptr::null_mut(),
        ))?;
        Ok(value as u8)
    }
}

impl DAQmxInput<u8> for Task<DigitalInput> {
    unsafe fn daqmx_read(
        &mut self,
        samples_per_channel: i32,
        timeout: f64,
        fill_mode: daqmx::bool32,
        buffer: *mut u8,
        buffer_size: u32,
        actual_samples_per_channel: *mut i32,
    ) -> i32 {
        daqmx::DAQmxReadDigitalLines(
            self.raw_handle(),
            samples_per_channel,
            timeout,
            fill_mode,
            buffer,
            buffer_size,
            actual_samples_per_channel,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    }
}
