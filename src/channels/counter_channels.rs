use anyhow::bail;
use anyhow::Result;
use derive_builder::Builder;
use std::ffi::CString;

use super::{Channel, ChannelBuilderInput, ChannelBuilderOutput};
use crate::daqmx;
use crate::daqmx::*;
use crate::daqmx_call;
use crate::tasks::{CounterInput, CounterOutput, Task};
use crate::types::{ClockEdge, CountDirection, FrequencyUnits, IdleState, TimeUnits};

pub trait CounterChannelType {}

impl CounterChannelType for CounterInput {}
impl CounterChannelType for CounterOutput {}

pub trait CounterChannelTrait<T: CounterChannelType>: Sized {
    fn new(task: Task<T>, name: &str) -> Result<Self>;
}

pub struct CounterChannelBase<T: CounterChannelType> {
    task: Task<T>,
    name: CString,
}

impl<T: CounterChannelType> CounterChannelTrait<T> for CounterChannelBase<T> {
    fn new(task: Task<T>, name: &str) -> Result<Self> {
        let name = CString::new(name)?;
        Ok(Self { task, name })
    }
}

impl<T: CounterChannelType> Channel for CounterChannelBase<T> {
    fn raw_handle(&self) -> *mut std::os::raw::c_void {
        self.task.raw_handle()
    }
    fn name(&self) -> &std::ffi::CStr {
        &self.name
    }
}

impl<T: CounterChannelType> CounterChannelBase<T> {
    pub fn physical_channel(&self) -> Result<String> {
        self.read_channel_property_string(daqmx::DAQmxGetPhysicalChanName)
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(setter(into))]
pub struct CounterOutputPulseTimeChannel {
    physical_counter: CString,
    #[builder(default)]
    name: Option<CString>,
    #[builder(default = "TimeUnits::Seconds")]
    pub units: TimeUnits,
    #[builder(default = "IdleState::Low")]
    pub idle_state: IdleState,
    #[builder(default = "0.0")]
    pub initial_delay: f64,
    #[builder(default = "0.001")]
    pub low_time: f64,
    #[builder(default = "0.001")]
    pub high_time: f64,
}

impl CounterOutputPulseTimeChannel {
    pub fn builder<N: AsRef<str>, P: AsRef<str>>(
        name: N,
        physical_counter: P,
    ) -> Result<CounterOutputPulseTimeChannelBuilder> {
        let physical_counter = CString::new(physical_counter.as_ref())?;
        let mut builder = CounterOutputPulseTimeChannelBuilder::default();
        builder.physical_counter(physical_counter);
        builder.name(CString::new(name.as_ref())?);
        Ok(builder)
    }
}

impl ChannelBuilderOutput for CounterOutputPulseTimeChannel {
    fn add_to_task(self, task: TaskHandle) -> Result<()> {
        if self.low_time <= 0.0 {
            bail!("low_time must be > 0.0 seconds");
        }
        if self.high_time <= 0.0 {
            bail!("high_time must be > 0.0 seconds");
        }
        if self.initial_delay < 0.0 {
            bail!("initial_delay must be >= 0.0 seconds");
        }

        let empty_string = CString::default();
        daqmx_call!(daqmx::DAQmxCreateCOPulseChanTime(
            task,
            self.physical_counter.as_ptr(),
            self.name.as_ref().unwrap_or(&empty_string).as_ptr(),
            self.units.into(),
            self.idle_state.into(),
            self.initial_delay,
            self.low_time,
            self.high_time
        ))
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(setter(into))]
pub struct CounterOutputPulseFreqChannel {
    physical_counter: CString,
    #[builder(default)]
    name: Option<CString>,
    #[builder(default = "FrequencyUnits::Hertz")]
    pub units: FrequencyUnits,
    #[builder(default = "IdleState::Low")]
    pub idle_state: IdleState,
    #[builder(default = "0.0")]
    pub initial_delay: f64,
    #[builder(default = "1000.0")]
    pub frequency: f64,
    #[builder(default = "0.5")]
    pub duty_cycle: f64,
}

impl CounterOutputPulseFreqChannel {
    pub fn builder<N: AsRef<str>, P: AsRef<str>>(
        name: N,
        physical_counter: P,
    ) -> Result<CounterOutputPulseFreqChannelBuilder> {
        let physical_counter = CString::new(physical_counter.as_ref())?;
        let mut builder = CounterOutputPulseFreqChannelBuilder::default();
        builder.physical_counter(physical_counter);
        builder.name(CString::new(name.as_ref())?);
        Ok(builder)
    }
}

impl ChannelBuilderOutput for CounterOutputPulseFreqChannel {
    fn add_to_task(self, task: TaskHandle) -> Result<()> {
        if self.frequency <= 0.0 {
            bail!("frequency must be > 0.0 Hz");
        }
        if !(0.0 < self.duty_cycle && self.duty_cycle < 1.0) {
            bail!("duty_cycle must be in range (0.0, 1.0)");
        }
        if self.initial_delay < 0.0 {
            bail!("initial_delay must be >= 0.0 seconds");
        }

        let empty_string = CString::default();
        daqmx_call!(daqmx::DAQmxCreateCOPulseChanFreq(
            task,
            self.physical_counter.as_ptr(),
            self.name.as_ref().unwrap_or(&empty_string).as_ptr(),
            self.units.into(),
            self.idle_state.into(),
            self.initial_delay,
            self.frequency,
            self.duty_cycle
        ))
    }
}

#[derive(Builder, Debug, Clone)]
#[builder(setter(into))]
pub struct CounterInputCountEdgesChannel {
    physical_counter: CString,
    #[builder(default)]
    name: Option<CString>,
    #[builder(default = "ClockEdge::Rising")]
    pub edge: ClockEdge,
    #[builder(default = "0")]
    pub initial_count: u32,
    #[builder(default = "CountDirection::CountUp")]
    pub count_direction: CountDirection,
}

impl CounterInputCountEdgesChannel {
    pub fn builder<N: AsRef<str>, P: AsRef<str>>(
        name: N,
        physical_counter: P,
    ) -> Result<CounterInputCountEdgesChannelBuilder> {
        let physical_counter = CString::new(physical_counter.as_ref())?;
        let mut builder = CounterInputCountEdgesChannelBuilder::default();
        builder.physical_counter(physical_counter);
        builder.name(CString::new(name.as_ref())?);
        Ok(builder)
    }
}

impl ChannelBuilderInput for CounterInputCountEdgesChannel {
    fn add_to_task(self, task: TaskHandle) -> Result<()> {
        let empty_string = CString::default();
        daqmx_call!(daqmx::DAQmxCreateCICountEdgesChan(
            task,
            self.physical_counter.as_ptr(),
            self.name.as_ref().unwrap_or(&empty_string).as_ptr(),
            self.edge.into(),
            self.initial_count,
            self.count_direction.into()
        ))
    }
}
