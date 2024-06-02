use crate::channels::{ChannelBuilderOutput, DigitalChannelTrait};
use crate::types::Timeout;
use crate::{daqmx, daqmx_call};
use anyhow::Result;
use std::ptr;

use super::output::{DAQmxOutput, OutputTask};
use super::{task::DigitalOutput, Task};

impl Task<DigitalOutput> {
    pub fn create_channel<B: ChannelBuilderOutput>(&mut self, builder: B) -> Result<()> {
        builder.add_to_task(self.raw_handle())
    }

    pub fn get_channel<C: DigitalChannelTrait<DigitalOutput>>(&self, name: &str) -> Result<C> {
        C::new(self.clone(), name)
    }
}

impl OutputTask<u8> for Task<DigitalOutput> {
    fn write_scalar(&mut self, value: u8, timeout: Timeout) -> Result<()> {
        daqmx_call!(daqmx::DAQmxWriteDigitalScalarU32(
            self.raw_handle(),
            1,
            timeout.into(),
            value as u32,
            ptr::null_mut()
        ))?;
        Ok(())
    }
}

impl DAQmxOutput<u8> for Task<DigitalOutput> {
    unsafe fn daqmx_write(
        &mut self,
        samples_per_channel: i32,
        timeout: f64,
        fill_mode: daqmx::bool32,
        buffer: *const u8,
        actual_samples_per_channel: *mut i32,
    ) -> i32 {
        let autostart = daqmx::bool32::from(false); // TODO: investigate autostart

        daqmx::DAQmxWriteDigitalLines(
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
