use anyhow::Result;
use daqmx::tasks::AnalogInput;
use serial_test::serial;
use std::ffi::CString;

use daqmx::channels::*;
use daqmx::scales::LinearScale;
use daqmx::scales::PreScaledUnits;
use daqmx::tasks::InputTask;
use daqmx::tasks::Task;
use daqmx::types::ClockEdge;
use daqmx::types::ClockEdge::Rising;
use daqmx::types::DataFillMode;
use daqmx::types::SampleMode;
use daqmx::types::Timeout;

#[test]
#[serial]
fn test_voltage_input_builder() -> Result<()> {
    let ch1 = VoltageChannel::new("my name", "PCIe-6363_test/ai1")?
        .max(1.0)
        .terminal_config(AnalogTerminalConfig::RSE)
        .build()?;

    let mut task: Task<AnalogInput> = Task::new("")?;
    task.create_channel(ch1)?;

    let configured: VoltageChannelBase<AnalogInput> = task.get_channel("my name")?;

    assert_eq!(
        configured.physical_channel()?,
        "PCIe-6363_test/ai1".to_owned()
    );
    assert_eq!(configured.ai_max()?, 1.0);
    assert_eq!(configured.ai_min()?, -1.0);
    assert_eq!(configured.ai_terminal_config()?, AnalogTerminalConfig::RSE);
    assert_eq!(configured.scale()?, VoltageScale::Volts);

    task.start()?;

    const SAMPLES: usize = 10;
    let mut buffer = [0.0; SAMPLES];

    let scalar = task.read_scalar(Timeout::Seconds(10.0))?;
    assert_ne!(scalar, 0.0);
    task.stop()?;

    task.configure_sample_clock_timing(
        None,
        100.0,
        Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;
    task.start()?;

    task.read(
        Timeout::Seconds(1.0),
        DataFillMode::GroupByChannel,
        None,
        &mut buffer,
    )?;

    task.stop()?;
    Ok(())
}

#[test]
#[serial]
fn test_scalar_read() -> Result<()> {
    let mut task: Task<AnalogInput> = Task::new("scalar")?;
    let ch1 = VoltageChannel::new("my name", "PCIe-6363_test/ai0")?.build()?;
    task.create_channel(ch1)?;
    let res = task.read_scalar(Timeout::Seconds(1.0))?;
    assert_ne!(res, 0.0);
    drop(task);
    Ok(())
}

#[test]
#[serial]
fn test_buffered_read() -> Result<()> {
    let mut task: Task<AnalogInput> = Task::new("scalar")?;
    let ch1 = VoltageChannel::new("my_name", "PCIe-6363_test/ai0")?.build()?;
    task.create_channel(ch1)?;
    task.configure_sample_clock_timing(
        None,
        1000.0,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        100,
    )?;

    let mut buffer = [0.0; 100];

    task.start()?;
    task.read(
        Timeout::Seconds(1.0),
        DataFillMode::GroupByChannel,
        Some(100),
        &mut buffer[..],
    )?;

    Ok(())
}

#[test]
#[serial]
fn test_buffered_read_with_timeout() -> Result<()> {
    let mut task: Task<AnalogInput> = Task::new("scalar")?;
    let ch1 = VoltageChannel::new("my_name", "PCIe-6363_test/ai0")?.build()?;
    task.create_channel(ch1)?;
    task.configure_sample_clock_timing(
        None,
        10.0,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        100,
    )?;

    let mut buffer = [0.0; 100];

    task.start()?;
    let error = task
        .read(
            Timeout::Seconds(1.0),
            DataFillMode::GroupByChannel,
            Some(100),
            &mut buffer[..],
        )
        .is_err();

    assert_eq!(error, true);

    Ok(())
}

#[test]
#[serial]
fn test_stop() -> Result<()> {
    let mut task: Task<AnalogInput> = Task::new("scalar")?;
    let ch1 = VoltageChannel::new("my_name", "PCIe-6363_test/ai0")?.build()?;
    task.create_channel(ch1)?;
    task.configure_sample_clock_timing(
        None,
        1000.0,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        100,
    )?;

    let mut buffer = [0.0; 100];

    task.set_read_auto_start(false)?;
    task.start()?;
    task.read(
        Timeout::Seconds(1.0),
        DataFillMode::GroupByChannel,
        Some(100),
        &mut buffer[..],
    )?;

    //now stop and confirm read response.
    task.stop()?;
    let read_result = task.read(
        Timeout::Seconds(1.0),
        DataFillMode::GroupByChannel,
        Some(100),
        &mut buffer[..],
    );

    assert_eq!(read_result.is_err(), true);

    if let Some(daqmx::error::DaqmxError::DaqmxError(code, _)) = read_result
        .unwrap_err()
        .downcast_ref::<daqmx::error::DaqmxError>(
    ) {
        assert_eq!(*code, -200473);
    } else {
        panic!("Expected DaqmxError with code -200473");
    }
    Ok(())
}

#[test]
#[serial]
fn test_voltage_input_builder_custom_scale() -> Result<()> {
    // create custom scale first.
    let _scale = LinearScale::new("TestScale", 1.0, 1.5, PreScaledUnits::Volts, "test")?;
    let ch1 = VoltageChannel::new("my name", "PCIe-6363_test/ai1")?
        .scale(VoltageScale::CustomScale(Some(
            CString::new("TestScale").expect("Name Error"),
        )))
        .max(5.0)
        .terminal_config(AnalogTerminalConfig::RSE)
        .build()?;

    let mut task: Task<AnalogInput> = Task::new("custom scale")?;
    task.create_channel(ch1)?;

    let configured: VoltageChannelBase<AnalogInput> = task.get_channel("my name")?;

    assert_eq!(
        configured.scale()?,
        VoltageScale::CustomScale(Some(CString::new("TestScale").expect("Name Error")))
    );

    task.start()?;

    let mut buffer = [0.0; 10];
    task.read(
        Timeout::Seconds(1.0),
        DataFillMode::GroupByChannel,
        Some(10),
        &mut buffer,
    )?;

    assert_ne!(buffer[0], 0.0);
    Ok(())
}
