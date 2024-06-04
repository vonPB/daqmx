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
        buffer: Vec<u8>,
        actual_samples_per_channel: *mut i32,
    ) -> i32 {
        let autostart = daqmx::bool32::from(true);
        let mut written = 0;

        // let temp_buffer = buffer.clone();
        // let buffer = temp_buffer;

        let buffer = [
            1u8, 0u8, 1u8, // Channel 0: 3 samples
            1u8, 0u8, 1u8, // Channel 1: 3 samples
            1u8, 0u8, 1u8, // Channel 2: 3 samples
        ];

        let res = daqmx::DAQmxWriteDigitalLines(
            self.raw_handle(),
            samples_per_channel,
            autostart,
            timeout,
            fill_mode,
            buffer.as_ptr(),
            &mut written,
            ptr::null_mut(),
        );
        // *actual_samples_per_channel = written;
        res
    }
}

// impl DAQmxOutput<u8> for Task<DigitalOutput> {
//     unsafe fn daqmx_write(
//         &mut self,
//         samples_per_channel: i32,
//         timeout: f64,
//         fill_mode: daqmx::bool32,
//         buffer: Vec<u8>,
//         actual_samples_per_channel: *mut i32,
//     ) -> i32 {
//         let autostart = daqmx::bool32::from(true);
//
//         // let x = Box::from_raw(buffer as *mut [u8; 9]);
//         // println!("Buffer raw: {:?}", x);
//         //
//         // let buffer = [
//         //     1u8, 0u8, 1u8, // Channel 0: 3 samples
//         //     1u8, 0u8, 1u8, // Channel 1: 3 samples
//         //     1u8, 0u8, 1u8, // Channel 2: 3 samples
//         // ];
//         // let buffer = buffer.as_ptr();
//         //
//         // let y = Box::from_raw(buffer as *mut [u8; 9]);
//         // println!("Local raw: {:?}", y);
//         //
//         // println!("Equal: {:?}", x == y);
//
//         let mut written = 0;
//         // let array = buffer.as_slice();
//         // std::mem::forget(array);
//
//         let buffer = vec![
//             1u8, 0u8, 1u8, // Channel 0: 3 samples
//             1u8, 0u8, 1u8, // Channel 1: 3 samples
//             1u8, 0u8, 1u8, // Channel 2: 3 samples
//         ];
//
//         let ar = buffer.as_slice();
//
//         let res = daqmx::DAQmxWriteDigitalLines(
//             self.raw_handle(),
//             samples_per_channel,
//             autostart,
//             timeout,
//             fill_mode,
//             ar.as_ptr(),
//             &mut written,
//             ptr::null_mut(),
//         );
//         // *actual_samples_per_channel = written;
//
//         res
//     }
// }
