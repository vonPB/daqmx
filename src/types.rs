// This file contains code derived from the daqmx-rs project:
// https://github.com/WiresmithTech/daqmx-rs

use crate::daqmx;

/// Module for handling FFI interface types and general DAQmx Types.
/// The FFI exposes the char* interface as i8 and requires preallocation in a way
/// that CString doesn't like for string outputs.
///
/// This function will strip the end null out of the buffer and format to a string.
pub(crate) fn buffer_to_string(buffer: Vec<i8>) -> String {
    // First get just valid chars as u8
    let buffer_u8 = buffer
        .into_iter()
        .take_while(|&e| e != 0)
        .map(|e| e as u8)
        .collect();

    // Build from utf8 - I think it may be ascii but should still be compliant as utf8.
    // In the Python API this is treated as UTF8 as well.
    String::from_utf8(buffer_u8).expect("Invalid Characters in Error Buffer")
}

/// Describes the memory layout of a 1D buffer that represents 2D data.
///
/// This will impact the access patterns when you read the data which can impact performance.
pub enum DataFillMode {
    /// The layout groups data by channel. i.e. [Channel 0 Sample 0-2, Channel 1 Sample 0-2]
    /// Also known as noninterleaved.
    GroupByChannel,
    /// The layout groups data by sample.  i.e. [Sample 0 Channel 0-1, Sample 1 Channel 0-1, Sample 2 Channel 0-1]
    /// Also known as interleaved.
    GroupByScanNumber,
}

impl From<DataFillMode> for daqmx::bool32 {
    fn from(fill_mode: DataFillMode) -> Self {
        match fill_mode {
            DataFillMode::GroupByChannel => daqmx::DAQmx_Val_GroupByChannel as u32,
            DataFillMode::GroupByScanNumber => daqmx::DAQmx_Val_GroupByScanNumber as u32,
        }
    }
}

/// Enum representing the timeout options in the read and write APIs.
pub enum Timeout {
    /// Wait forever for the samples to become available.
    WaitForever,
    /// Immediately read the samples.
    NoWait,
    /// Time in seconds to wait for the requested samples to become available.
    /// If not all samples are available then whatever samples are available are read and an error is returned
    Seconds(f64),
}

impl From<Timeout> for f64 {
    fn from(timeout: Timeout) -> Self {
        match timeout {
            Timeout::WaitForever => daqmx::DAQmx_Val_WaitInfinitely,
            Timeout::NoWait => 0.0,
            Timeout::Seconds(seconds) => seconds,
        }
    }
}

///Represents the active edge of clock or trigger
///
/// Default is rising.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockEdge {
    Rising,
    Falling,
}

impl Default for ClockEdge {
    fn default() -> Self {
        Self::Rising
    }
}

impl From<ClockEdge> for i32 {
    fn from(edge: ClockEdge) -> Self {
        match edge {
            ClockEdge::Rising => daqmx::DAQmx_Val_Rising,
            ClockEdge::Falling => daqmx::DAQmx_Val_Falling,
        }
    }
}

/// Idle state for counter output pulses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IdleState {
    #[default]
    Low,
    High,
}

impl From<IdleState> for i32 {
    fn from(idle: IdleState) -> Self {
        match idle {
            IdleState::Low => daqmx::DAQmx_Val_Low,
            IdleState::High => daqmx::DAQmx_Val_High,
        }
    }
}

/// Time units used by pulse-time counter channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TimeUnits {
    #[default]
    Seconds,
}

impl From<TimeUnits> for i32 {
    fn from(units: TimeUnits) -> Self {
        match units {
            TimeUnits::Seconds => daqmx::DAQmx_Val_Seconds,
        }
    }
}

/// Frequency units used by pulse-frequency counter channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FrequencyUnits {
    #[default]
    Hertz,
}

impl From<FrequencyUnits> for i32 {
    fn from(units: FrequencyUnits) -> Self {
        match units {
            FrequencyUnits::Hertz => daqmx::DAQmx_Val_Hz,
        }
    }
}

/// Count direction for edge-counting counter input channels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CountDirection {
    #[default]
    CountUp,
    CountDown,
    ExternalControl,
}

impl From<CountDirection> for i32 {
    fn from(direction: CountDirection) -> Self {
        match direction {
            CountDirection::CountUp => daqmx::DAQmx_Val_CountUp,
            CountDirection::CountDown => daqmx::DAQmx_Val_CountDown,
            CountDirection::ExternalControl => daqmx::DAQmx_Val_ExtControlled,
        }
    }
}

/// Signals that can be exported from a task onto a terminal.
///
/// Typical terminals:
/// - `"/DevX/PFI0"`
/// - `"/DevX/RTSI0"`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportSignal {
    /// Generic sample clock export (works with task sample clock).
    SampleClock,
    /// Generic start trigger export.
    StartTrigger,
    /// Generic reference trigger export.
    ReferenceTrigger,
    /// Generic arm-start trigger export.
    ArmStartTrigger,
    /// AI convert clock export.
    AiConvertClock,
    /// Generic counter output event export.
    CounterOutputEvent,
}

impl From<ExportSignal> for i32 {
    fn from(signal: ExportSignal) -> Self {
        match signal {
            ExportSignal::SampleClock => daqmx::DAQmx_Val_SampleClock,
            ExportSignal::StartTrigger => daqmx::DAQmx_Val_StartTrigger,
            ExportSignal::ReferenceTrigger => daqmx::DAQmx_Val_ReferenceTrigger,
            ExportSignal::ArmStartTrigger => daqmx::DAQmx_Val_ArmStartTrigger,
            ExportSignal::AiConvertClock => daqmx::DAQmx_Val_AIConvertClock,
            ExportSignal::CounterOutputEvent => daqmx::DAQmx_Val_CounterOutputEvent,
        }
    }
}

/// Represents the different timing modes of a task.
pub enum SampleMode {
    /// Acquire or generate a finite number of samples.
    FiniteSamples,
    /// Acquire or generate samples until you stop the task.
    ContinuousSamples,
    /// Acquire or generate samples continuously using hardware timing without a buffer.
    /// Hardware timed single point sample mode is supported only for the sample clock and change detection timing types.
    HardwareTimedSinglePoint,
}

impl From<SampleMode> for i32 {
    fn from(mode: SampleMode) -> Self {
        match mode {
            SampleMode::FiniteSamples => daqmx::DAQmx_Val_FiniteSamps,
            SampleMode::ContinuousSamples => daqmx::DAQmx_Val_ContSamps,
            SampleMode::HardwareTimedSinglePoint => daqmx::DAQmx_Val_HWTimedSinglePoint,
        }
    }
}

//Used quite a bit so lets re-export here with conversion.
pub use daqmx::bool32;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_buffer_to_string_good() {
        let buffer: Vec<i8> = vec![68, 101, 118, 105, 99, 101, 32, 105, 100, 101, 110, 0, 0, 0];
        let string = buffer_to_string(buffer);
        assert_eq!(&string, "Device iden");
    }

    #[test]
    fn test_error_buffer_to_string_no_null() {
        let buffer: Vec<i8> = vec![68, 101, 118, 105, 99, 101, 32, 105, 100, 101, 110];
        let string = buffer_to_string(buffer);
        assert_eq!(&string, "Device iden");
    }

    #[test]
    fn timeout_conversion_tests() {
        assert_eq!(
            f64::from(Timeout::WaitForever),
            daqmx::DAQmx_Val_WaitInfinitely
        );

        assert_eq!(f64::from(Timeout::NoWait), 0.0);

        assert_eq!(f64::from(Timeout::Seconds(2.1)), 2.1);
    }

    #[test]
    fn edge_conversion_tests() {
        assert_eq!(i32::from(ClockEdge::Rising), daqmx::DAQmx_Val_Rising);
        assert_eq!(i32::from(ClockEdge::Falling), daqmx::DAQmx_Val_Falling);
    }

    #[test]
    fn sample_mode_conversion_tests() {
        assert_eq!(
            i32::from(SampleMode::FiniteSamples),
            daqmx::DAQmx_Val_FiniteSamps
        );
        assert_eq!(
            i32::from(SampleMode::ContinuousSamples),
            daqmx::DAQmx_Val_ContSamps
        );
        assert_eq!(
            i32::from(SampleMode::HardwareTimedSinglePoint),
            daqmx::DAQmx_Val_HWTimedSinglePoint
        );
    }
}
