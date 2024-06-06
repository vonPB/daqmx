use std::ffi::CString;

use crate::daqmx_call;
use anyhow::Result;
/// # Purpose
/// Immediately aborts all tasks associated with a device and returns the device to an initialized state.
/// Aborting a task stops and releases any resources the task reserved.
///
/// This function does not wait for the device to reset before continuing execution.
/// When resetting a chassis, attached modules are unavailable.
/// If you attempt to reset an attached module during this time, you will receive an error.
pub fn reset_device(device: &str) -> Result<()> {
    let c_device = CString::new(device)?;
    let d_ptr = c_device.as_ptr();

    daqmx_call!(crate::daqmx::DAQmxResetDevice(d_ptr))?;

    Ok(())
}

mod info {
    use anyhow::Result;
    use std::ffi::CString;

    use crate::{
        daqmx::{
            DAQmxGetDevAIPhysicalChans, DAQmxGetDevAOPhysicalChans, DAQmxGetDevCIPhysicalChans,
            DAQmxGetDevCOPhysicalChans, DAQmxGetDevDILines, DAQmxGetDevDOLines,
        },
        daqmx_call,
        types::buffer_to_string,
    };

    #[cfg_attr(
        feature = "serde_support",
        derive(serde::Serialize, serde::Deserialize)
    )]
    pub enum ChannelType {
        AI, // Analog Input
        AO, // Analog Output
        DI, // Digital Input
        DO, // Digital Output
        CI, // Counter Input
        CO, // Counter Output
    }

    pub fn get_channels(
        device: &str,
        channel_type: ChannelType,
        trim_device_name: bool,
    ) -> Result<Vec<String>> {
        let mut buf = vec![0i8; 5120];
        let c_device = CString::new(device)?;

        let b_ptr = buf.as_mut_ptr();
        let d_ptr = c_device.as_ptr();

        match channel_type {
            ChannelType::DI => daqmx_call!(DAQmxGetDevDILines(d_ptr, b_ptr, 5120))?,
            ChannelType::DO => daqmx_call!(DAQmxGetDevDOLines(d_ptr, b_ptr, 5120))?,
            ChannelType::AI => daqmx_call!(DAQmxGetDevAIPhysicalChans(d_ptr, b_ptr, 5120))?,
            ChannelType::AO => daqmx_call!(DAQmxGetDevAOPhysicalChans(d_ptr, b_ptr, 5120))?,
            ChannelType::CI => daqmx_call!(DAQmxGetDevCIPhysicalChans(d_ptr, b_ptr, 5120))?,
            ChannelType::CO => daqmx_call!(DAQmxGetDevCOPhysicalChans(d_ptr, b_ptr, 5120))?,
        };

        let buffer = buffer_to_string(buf);

        let channels: Vec<String> = buffer
            .lines()
            .flat_map(|line| line.split(", "))
            .map(|s| {
                if trim_device_name {
                    s.replace(&format!("{}/", device), "")
                } else {
                    s.to_string()
                }
            })
            .collect();

        Ok(channels)
    }

    #[test]
    #[serial_test::serial]
    fn test_get_channels() -> Result<()> {
        {
            let res = get_channels("PCIe-6363_test", ChannelType::AI, false)?;
            assert!(res.len() > 0);
        }
        {
            let res = get_channels("PCIe-6363_test", ChannelType::AO, false)?;
            assert!(res.len() > 0);
        }
        {
            let res = get_channels("PCIe-6363_test", ChannelType::DI, true)?;
            assert!(res.len() > 0);
        }
        {
            let res = get_channels("PCIe-6363_test", ChannelType::DO, true)?;
            assert!(res.len() > 0);
        }
        Ok(())
    }
}
