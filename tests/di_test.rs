use anyhow::Result;
use daqmx::channels::*;
use daqmx::tasks::DigitalInput;
use daqmx::tasks::InputTask;
use daqmx::tasks::Task;
use daqmx::types::ClockEdge::Rising;
use daqmx::types::DataFillMode;
use daqmx::types::SampleMode;
use daqmx::types::Timeout;
use serial_test::serial;

#[test]
#[serial]
fn test_digital_input_builder_scalar_types() -> Result<()> {
    let ch1 = DigitalChannel::new("my_digital_input", "PCIe-6363_test/port0/line0")?.build()?;

    let mut task: Task<DigitalInput> = Task::new("")?;
    task.create_channel(ch1)?;

    let _res: bool = task.read_scalar(Timeout::Seconds(0.1))?;
    let _res: u8 = task.read_scalar(Timeout::Seconds(0.1))?;

    task.stop()?;
    Ok(())
}

#[test]
#[serial]
fn test_digital_input_builder() -> Result<()> {
    let ch1 = DigitalChannel::new("my_digital_input", "PCIe-6363_test/port0/line0")?.build()?;

    let ch2 = DigitalChannel::new(
        "my_digital_input2",
        "PCIe-6363_test/port0/line1, PCIe-6363_test/port0/line2",
    )?
    .build()?;

    let mut task: Task<DigitalInput> = Task::new("")?;
    task.create_channel(ch1)?;
    task.create_channel(ch2)?;

    task.configure_sample_clock_timing(None, 100.0, Rising, SampleMode::FiniteSamples, 10 as u64)?;

    let configured: DigitalChannelBase<DigitalInput> = task.get_channel("my_digital_input")?;

    assert_eq!(
        configured.physical_channel()?,
        "PCIe-6363_test/port0/line0".to_owned()
    );

    task.start()?;
    let mut buffer = [0u8; 40];

    task.read(
        Timeout::Seconds(10.0),
        DataFillMode::GroupByChannel,
        Some(10),
        &mut buffer,
    )?;

    task.stop()?;
    Ok(())
}

#[test]
#[serial]
fn test_digital_input_builder_bool() -> Result<()> {
    let ch1 = DigitalChannel::new("my_digital_input", "PCIe-6363_test/port0/line0")?.build()?;

    let ch2 = DigitalChannel::new(
        "my_digital_input2",
        "PCIe-6363_test/port0/line1, PCIe-6363_test/port0/line2",
    )?
    .build()?;

    let mut task: Task<DigitalInput> = Task::new("")?;
    task.create_channel(ch1)?;
    task.create_channel(ch2)?;

    task.configure_sample_clock_timing(None, 100.0, Rising, SampleMode::FiniteSamples, 10 as u64)?;

    let configured: DigitalChannelBase<DigitalInput> = task.get_channel("my_digital_input")?;

    assert_eq!(
        configured.physical_channel()?,
        "PCIe-6363_test/port0/line0".to_owned()
    );

    task.start()?;
    let mut buffer = [false; 40];

    task.read(
        Timeout::Seconds(10.0),
        DataFillMode::GroupByChannel,
        Some(10),
        &mut buffer,
    )?;

    task.stop()?;
    Ok(())
}
