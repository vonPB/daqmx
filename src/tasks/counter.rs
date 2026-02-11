use std::{ffi::CString, ptr};

use anyhow::bail;
use anyhow::Result;

use crate::channels::{ChannelBuilderInput, ChannelBuilderOutput, CounterChannelTrait};
use crate::types::{ExportSignal, IdleState, SampleMode, Timeout};
use crate::{daqmx, daqmx_call};

use super::{CounterInput, CounterOutput, Task};

/// Counter output task operations.
///
/// Counter-generated pulses are hardware timed and are preferred for deterministic triggering
/// over software-timed digital outputs.
pub trait CounterOutputTask {
    /// Configure implicit timing for counter output generation.
    fn configure_implicit_timing(
        &mut self,
        mode: SampleMode,
        samples_per_channel: u64,
    ) -> Result<()>;

    /// Start a prepared pulse generation task.
    fn start_pulse(&mut self) -> Result<()>;
}

/// Counter input task operations for basic edge count and period reads.
pub trait CounterInputTask {
    /// Read scalar edge count.
    fn read_count_scalar(&mut self, timeout: Timeout) -> Result<u32>;

    /// Read scalar period measurement (seconds).
    fn read_period_scalar(&mut self, timeout: Timeout) -> Result<f64>;
}

impl Task<CounterOutput> {
    pub fn create_channel<B: ChannelBuilderOutput>(&mut self, builder: B) -> Result<()> {
        builder.add_to_task(self.raw_handle())?;
        self.channel_count += 1;
        Ok(())
    }

    pub fn get_channel<C: CounterChannelTrait<CounterOutput>>(&self, name: &str) -> Result<C> {
        C::new(self.clone(), name)
    }

    /// Configure a one-shot pulse-time counter output using implicit finite timing (1 sample).
    ///
    /// `counter` is a physical counter such as `"Dev1/ctr0"`.
    pub fn configure_one_shot_pulse_time(
        &mut self,
        counter: &str,
        low_s: f64,
        high_s: f64,
        idle_state: IdleState,
    ) -> Result<()> {
        if low_s <= 0.0 {
            bail!("low_s must be > 0.0 seconds");
        }
        if high_s <= 0.0 {
            bail!("high_s must be > 0.0 seconds");
        }

        let counter_c = CString::new(counter)?;
        daqmx_call!(daqmx::DAQmxCreateCOPulseChanTime(
            self.raw_handle(),
            counter_c.as_ptr(),
            ptr::null(),
            daqmx::DAQmx_Val_Seconds,
            idle_state.into(),
            0.0,
            low_s,
            high_s
        ))?;
        self.channel_count += 1;
        self.configure_implicit_timing(SampleMode::FiniteSamples, 1)
    }

    /// Export this task's counter output event to a terminal (for example `"/Dev1/PFI0"`).
    ///
    /// This is a convenience wrapper around:
    /// `export_signal(ExportSignal::CounterOutputEvent, terminal)`.
    pub fn export_counter_output_event_to(&mut self, terminal: &str) -> Result<()> {
        self.export_signal(ExportSignal::CounterOutputEvent, terminal)
    }

    /// Set the pulse output terminal for all counter output channels in this task.
    ///
    /// Typical terminals:
    /// - `"/DevX/PFI0"`
    /// - `"/DevX/PFI1"`
    ///
    /// This is often more direct than exporting a signal when you're routing only the
    /// counter pulse itself.
    pub fn set_counter_output_terminal(&mut self, terminal: &str) -> Result<()> {
        let empty_channel = CString::new("")?;
        let terminal_c = CString::new(terminal)?;
        daqmx_call!(daqmx::DAQmxSetCOPulseTerm(
            self.raw_handle(),
            empty_channel.as_ptr(),
            terminal_c.as_ptr()
        ))
    }

    /// Set the pulse output terminal for a specific counter output channel in this task.
    pub fn set_counter_output_terminal_for_channel(
        &mut self,
        channel_name: &str,
        terminal: &str,
    ) -> Result<()> {
        let channel_c = CString::new(channel_name)?;
        let terminal_c = CString::new(terminal)?;
        daqmx_call!(daqmx::DAQmxSetCOPulseTerm(
            self.raw_handle(),
            channel_c.as_ptr(),
            terminal_c.as_ptr()
        ))
    }
}

impl CounterOutputTask for Task<CounterOutput> {
    fn configure_implicit_timing(
        &mut self,
        mode: SampleMode,
        samples_per_channel: u64,
    ) -> Result<()> {
        daqmx_call!(daqmx::DAQmxCfgImplicitTiming(
            self.raw_handle(),
            mode.into(),
            samples_per_channel
        ))
    }

    fn start_pulse(&mut self) -> Result<()> {
        self.start()
    }
}

impl Task<CounterInput> {
    pub fn create_channel<B: ChannelBuilderInput>(&mut self, builder: B) -> Result<()> {
        builder.add_to_task(self.raw_handle())?;
        self.channel_count += 1;
        Ok(())
    }

    pub fn get_channel<C: CounterChannelTrait<CounterInput>>(&self, name: &str) -> Result<C> {
        C::new(self.clone(), name)
    }
}

impl CounterInputTask for Task<CounterInput> {
    fn read_count_scalar(&mut self, timeout: Timeout) -> Result<u32> {
        let mut value = 0u32;
        daqmx_call!(daqmx::DAQmxReadCounterScalarU32(
            self.raw_handle(),
            timeout.into(),
            &mut value,
            ptr::null_mut()
        ))?;
        Ok(value)
    }

    fn read_period_scalar(&mut self, timeout: Timeout) -> Result<f64> {
        let mut value = 0.0f64;
        daqmx_call!(daqmx::DAQmxReadCounterScalarF64(
            self.raw_handle(),
            timeout.into(),
            &mut value,
            ptr::null_mut()
        ))?;
        Ok(value)
    }
}
