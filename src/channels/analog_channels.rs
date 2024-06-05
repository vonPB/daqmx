use delegate::delegate;
use std::ffi::CString;

use derive_builder::Builder;

use crate::daqmx;
use crate::daqmx::*;

use super::{Channel, ChannelBuilderInput, ChannelBuilderOutput};
use crate::daqmx_call;
use crate::error::DaqmxError;
use crate::scales::PreScaledUnits;
use crate::tasks::{AnalogInput, AnalogOutput, Task};
use anyhow::Result;

macro_rules! delegate_ai_channel {
    () => {
        delegate! {
                to self.ai_channel {
                    pub fn ai_max(&self) -> Result<f64>;
                    pub fn ai_min(&self) -> Result<f64>;
                    pub fn physical_channel(&self) -> Result<String>;
                    pub fn ai_terminal_config(&self) -> Result<AnalogTerminalConfig>;
                }
        }
    };
}

pub trait AnalogChannelType {}

impl AnalogChannelType for AnalogInput {}
impl AnalogChannelType for AnalogOutput {}

pub trait AnalogChannelTrait<T: AnalogChannelType>: Sized {
    fn new(task: Task<T>, name: &str) -> Result<Self>;
}

pub struct AnalogChannelBase<T: AnalogChannelType> {
    task: Task<T>,
    name: CString,
}

impl<T: AnalogChannelType> AnalogChannelTrait<T> for AnalogChannelBase<T> {
    fn new(task: Task<T>, name: &str) -> Result<Self> {
        let name = CString::new(name)?;
        Ok(Self { task, name })
    }
}

impl<T: AnalogChannelType> Channel for AnalogChannelBase<T> {
    fn raw_handle(&self) -> *mut std::os::raw::c_void {
        self.task.raw_handle()
    }

    fn name(&self) -> &std::ffi::CStr {
        &self.name
    }
}

impl<T: AnalogChannelType> AnalogChannelBase<T> {
    /// Not needed for [AnalogOutput] channels]
    pub fn physical_channel(&self) -> Result<String> {
        self.read_channel_property_string(daqmx::DAQmxGetPhysicalChanName)
    }
    pub fn ai_max(&self) -> Result<f64> {
        self.read_channel_property(daqmx::DAQmxGetAIMax)
    }
    pub fn ai_min(&self) -> Result<f64> {
        self.read_channel_property(daqmx::DAQmxGetAIMin)
    }
    pub fn ai_terminal_config(&self) -> Result<AnalogTerminalConfig> {
        self.read_channel_property(daqmx::DAQmxGetAITermCfg)?
            .try_into()
    }
    pub fn custom_scale_name(&self) -> Result<String> {
        self.read_channel_property_string(daqmx::DAQmxGetAICustomScaleName)
    }
}

pub struct VoltageChannelBase<T: AnalogChannelType> {
    ai_channel: AnalogChannelBase<T>,
}

impl<T: AnalogChannelType> VoltageChannelBase<T> {
    delegate_ai_channel!();
    pub fn scale(&self) -> Result<VoltageScale> {
        let scale: VoltageScale = self
            .ai_channel
            .read_channel_property(DAQmxGetAIVoltageUnits)?
            .try_into()?;

        if let VoltageScale::CustomScale(_) = scale {
            let name = self.ai_channel.custom_scale_name()?;
            Ok(VoltageScale::CustomScale(Some(CString::new(name)?)))
        } else {
            Ok(scale)
        }
    }
}

impl<T: AnalogChannelType> AnalogChannelTrait<T> for VoltageChannelBase<T> {
    fn new(task: Task<T>, name: &str) -> Result<Self> {
        let ai_channel = AnalogChannelBase::new(task, name)?;
        Ok(Self { ai_channel })
    }
}

#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Defines the input configuration for the analog input.
pub enum AnalogTerminalConfig {
    /// Uses the [default for the type/hardware combination](https://www.ni.com/docs/en-US/bundle/ni-daqmx-device-considerations/page/defaulttermconfig.html).
    Default = DAQmx_Val_Cfg_Default,
    /// Configures inputs for reference single ended (reference to AI GND)
    RSE = DAQmx_Val_RSE,
    /// Cofngures inputs for non-reference single ended (reference to AI SENSE)
    NRSE = DAQmx_Val_NRSE,
    /// Configures inputs for differential mode.
    Differential = DAQmx_Val_Diff,
    /// Configures inputs for pseudo-differential mode
    PseudoDifferential = DAQmx_Val_PseudoDiff,
}

impl Default for AnalogTerminalConfig {
    fn default() -> Self {
        AnalogTerminalConfig::Default
    }
}

impl TryFrom<i32> for AnalogTerminalConfig {
    type Error = anyhow::Error;

    fn try_from(value: i32) -> anyhow::Result<Self> {
        //The if statements look wierd but seemed like the best way for the type conversion to be combined.
        match value {
            DAQmx_Val_Cfg_Default => Ok(Self::Default),
            DAQmx_Val_RSE => Ok(Self::RSE),
            DAQmx_Val_NRSE => Ok(Self::NRSE),
            DAQmx_Val_Diff => Ok(Self::Differential),
            DAQmx_Val_PseudoDiff => Ok(Self::PseudoDifferential),
            _ => Err(anyhow::anyhow!(
                "AnalogTerminalConfig value {} not recognized",
                value
            )),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum VoltageScale {
    Volts,
    /// A custom scale is in use. If we have not determined the name yet then this contains `None`.
    /// If we have determined the name, it will be contained in the option.
    CustomScale(Option<CString>),
    /// Units are set from the TEDS configuration. This cas should be read only.
    FromTEDS,
}

impl From<VoltageScale> for i32 {
    fn from(scale: VoltageScale) -> Self {
        match scale {
            VoltageScale::Volts => PreScaledUnits::Volts as i32,
            VoltageScale::CustomScale(_) => DAQmx_Val_FromCustomScale,
            VoltageScale::FromTEDS => PreScaledUnits::FromTEDS as i32,
        }
    }
}

///For the scale name.
impl From<VoltageScale> for CString {
    fn from(scale: VoltageScale) -> Self {
        // review: should this actually error if not custom.
        match scale {
            VoltageScale::CustomScale(Some(name)) => name.clone(),
            _ => CString::default(),
        }
    }
}

impl TryFrom<i32> for VoltageScale {
    type Error = DaqmxError;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        //The if statements look wierd but seemed like the best way for the type conversion to be combined.
        match value {
            DAQmx_Val_Volts => Ok(Self::Volts),
            DAQmx_Val_FromCustomScale => Ok(Self::CustomScale(None)),
            DAQmx_Val_FromTEDS => Ok(Self::FromTEDS),
            _ => Err(DaqmxError::UnexpectedValue(
                "AnalogTerminalConfig".to_string(),
                value,
            )),
        }
    }
}

/// Marker trait for Analog Input channel builders so the task can adapt to the type.
pub trait AnalogChannelBuilderTrait: ChannelBuilderInput {}

#[derive(Builder, Debug, Clone)]
#[builder(setter(into))]
pub struct VoltageChannel {
    physical_channel: CString,
    name: Option<CString>,
    #[builder(default = "5.0")]
    pub max: f64,
    #[builder(default = "-5.0")]
    pub min: f64,
    #[builder(default = "VoltageScale::Volts")]
    pub scale: VoltageScale,
    #[builder(default = "AnalogTerminalConfig::Default")]
    pub terminal_config: AnalogTerminalConfig,
}

impl VoltageChannel {
    pub fn new<S: Into<Vec<u8>>>(name: S, physical_channel: S) -> Result<VoltageChannelBuilder> {
        let physical_channel = CString::new(physical_channel)?;
        let mut builder = VoltageChannelBuilder::default();
        builder.physical_channel(physical_channel);
        builder.name(CString::new(name.into())?);
        Ok(builder)
    }
}

impl ChannelBuilderInput for VoltageChannel {
    fn add_to_task(self, task: TaskHandle) -> Result<()> {
        let empty_string = CString::default();
        daqmx_call!(daqmx::DAQmxCreateAIVoltageChan(
            task,
            self.physical_channel.as_ptr(),
            self.name.as_ref().unwrap_or(&empty_string).as_ptr(),
            self.terminal_config as i32,
            self.min,
            self.max,
            self.scale.clone().into(),
            CString::from(self.scale).as_ptr(),
        ))
    }
}

impl ChannelBuilderOutput for VoltageChannel {
    fn add_to_task(self, task: TaskHandle) -> Result<()> {
        let empty_string = CString::default();
        daqmx_call!(daqmx::DAQmxCreateAOVoltageChan(
            task,
            self.physical_channel.as_ptr(),
            self.name.as_ref().unwrap_or(&empty_string).as_ptr(),
            self.min,
            self.max,
            self.scale.clone().into(),
            CString::from(self.scale).as_ptr(),
        ))
    }
}

impl AnalogChannelBuilderTrait for VoltageChannel {}
