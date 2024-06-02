use super::{Channel, ChannelBuilderInput, ChannelBuilderOutput};
use crate::daqmx;
use crate::daqmx::*;
use crate::daqmx_call;
use crate::tasks::DigitalOutput;
use crate::tasks::{DigitalInput, Task};
use anyhow::Result;
use derive_builder::Builder;
use std::ffi::CString;

pub trait DigitalChannelType {}

impl DigitalChannelType for DigitalInput {}
impl DigitalChannelType for DigitalOutput {}

pub trait DigitalChannelTrait<T: DigitalChannelType>: Sized {
    fn new(task: Task<T>, name: &str) -> Result<Self>;
}

pub struct DigitalChannelBase<T: DigitalChannelType> {
    task: Task<T>,
    name: CString,
}

impl<T: DigitalChannelType> DigitalChannelTrait<T> for DigitalChannelBase<T> {
    fn new(task: Task<T>, name: &str) -> Result<Self> {
        let name = CString::new(name)?;
        Ok(Self { task, name })
    }
}

impl<T: DigitalChannelType> Channel for DigitalChannelBase<T> {
    fn raw_handle(&self) -> *mut std::os::raw::c_void {
        self.task.raw_handle()
    }
    fn name(&self) -> &std::ffi::CStr {
        &self.name
    }
}

impl<T: DigitalChannelType> DigitalChannelBase<T> {
    pub fn physical_channel(&self) -> Result<String> {
        self.read_channel_property_string(daqmx::DAQmxGetPhysicalChanName)
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(setter(into))]
pub struct DigitalChannel {
    physical_channel: CString,
    #[builder(default)]
    name: Option<CString>,
}

impl DigitalChannel {
    pub fn new<S: Into<Vec<u8>>>(name: S, physical_channel: S) -> Result<DigitalChannelBuilder> {
        let physical_channel = CString::new(physical_channel)?;
        let mut builder = DigitalChannelBuilder::default();
        builder.physical_channel(physical_channel);
        builder.name(CString::new(name.into())?);
        Ok(builder)
    }
}

/// Digital Input impl
impl ChannelBuilderInput for DigitalChannel {
    fn add_to_task(self, task: TaskHandle) -> Result<()> {
        let empty_string = CString::default();
        daqmx_call!(daqmx::DAQmxCreateDIChan(
            task,
            self.physical_channel.as_ptr(),
            self.name.as_ref().unwrap_or(&empty_string).as_ptr(),
            daqmx::DAQmx_Val_ChanForAllLines
        ))
    }
}

/// Digital Output impl
impl ChannelBuilderOutput for DigitalChannel {
    fn add_to_task(self, task: TaskHandle) -> Result<()> {
        let empty_string = CString::default();
        daqmx_call!(daqmx::DAQmxCreateDOChan(
            task,
            self.physical_channel.as_ptr(),
            self.name.as_ref().unwrap_or(&empty_string).as_ptr(),
            daqmx::DAQmx_Val_ChanForAllLines
        ))
    }
}
