use std::ffi::CString;

use anyhow::Result;

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

    match channel_type {
        ChannelType::DI => daqmx_call!(DAQmxGetDevDILines(
            c_device.as_ptr(),
            buf.as_mut_ptr(),
            5120
        ))?,
        ChannelType::DO => daqmx_call!(DAQmxGetDevDOLines(
            c_device.as_ptr(),
            buf.as_mut_ptr(),
            5120
        ))?,
        ChannelType::AI => daqmx_call!(DAQmxGetDevAIPhysicalChans(
            c_device.as_ptr(),
            buf.as_mut_ptr(),
            5120
        ))?,
        ChannelType::AO => daqmx_call!(DAQmxGetDevAOPhysicalChans(
            c_device.as_ptr(),
            buf.as_mut_ptr(),
            5120
        ))?,
        ChannelType::CI => daqmx_call!(DAQmxGetDevCIPhysicalChans(
            c_device.as_ptr(),
            buf.as_mut_ptr(),
            5120
        ))?,
        ChannelType::CO => daqmx_call!(DAQmxGetDevCOPhysicalChans(
            c_device.as_ptr(),
            buf.as_mut_ptr(),
            5120
        ))?,
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
