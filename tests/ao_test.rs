use anyhow::Result;
use daqmx::tasks::output::OutputTask;
use daqmx::tasks::AnalogOutput;
use serial_test::serial;

use daqmx::channels::*;
use daqmx::tasks::Task;
use daqmx::types::ClockEdge::Rising;
use daqmx::types::DataFillMode;
use daqmx::types::SampleMode;
use daqmx::types::Timeout;

#[test]
#[serial]
fn test_ao_scalar() -> Result<()> {
    let ch1 = VoltageChannel::new("my name", "PCIe-6363_test/ao1")?
        .max(1.0)
        .min(-1.0)
        .build()?;

    let mut task: Task<AnalogOutput> = Task::new("AnalogOutTest")?;
    task.create_channel(ch1)?;

    task.write_scalar(0.0, Timeout::Seconds(1.0))?;

    task.stop()?;
    Ok(())
}

#[test]
#[serial]
fn test_ao_buffered() -> Result<()> {
    let ch1 = VoltageChannel::new("my name", "PCIe-6363_test/ao1")?
        .max(1.0)
        .min(-1.0)
        .build()?;

    let mut task: Task<AnalogOutput> = Task::new("AnalogOutTest")?;
    task.create_channel(ch1)?;

    task.configure_sample_clock_timing(None, 1000.0, Rising, SampleMode::FiniteSamples, 10 as u64)
        .unwrap();

    task.start()?;

    let mut buffer = [0.0; 100];
    for i in 0..100 {
        buffer[i] = i as f64 / 100.0;
    }

    let written = task.write(
        Timeout::Seconds(10.0),
        DataFillMode::GroupByChannel,
        Some(100),
        &buffer[..],
    )?;

    assert_eq!(written, 100);

    task.stop()?;
    Ok(())
}

#[test]
#[serial]
fn test_ao_buffered_multi_port() -> Result<()> {
    let ch1 = VoltageChannel::new("my name", "PCIe-6363_test/ao0, PCIe-6363_test/ao1")?
        .max(1.0)
        .min(-1.0)
        .build()?;

    let mut task: Task<AnalogOutput> = Task::new("AnalogOutTest")?;
    task.create_channel(ch1)?;

    task.configure_sample_clock_timing(None, 1000.0, Rising, SampleMode::FiniteSamples, 10 as u64)
        .unwrap();

    task.start()?;

    let mut buffer = vec![0.0; 100];
    for i in 0..100 {
        buffer[i] = i as f64 / 100.0;
    }
    // Duplicate the buffer for secodn channel
    buffer.extend(buffer.clone());

    let written = task.write(
        Timeout::Seconds(10.0),
        DataFillMode::GroupByChannel,
        Some(100),
        &buffer[..],
    )?;

    assert_eq!(written, 100);

    task.stop()?;
    Ok(())
}

#[test]
#[serial]
fn test_ao_buffered_multi_channel() -> Result<()> {
    let ch1 = VoltageChannel::new("my name", "PCIe-6363_test/ao0")?
        .max(1.0)
        .min(-1.0)
        .build()?;

    let ch2 = VoltageChannel::new("my name2", "PCIe-6363_test/ao1")?
        .max(1.0)
        .min(-1.0)
        .build()?;

    let mut task: Task<AnalogOutput> = Task::new("AnalogOutTest")?;
    task.create_channel(ch1)?;
    task.create_channel(ch2)?;

    task.configure_sample_clock_timing(None, 1000.0, Rising, SampleMode::FiniteSamples, 10 as u64)
        .unwrap();

    task.start()?;

    let mut buffer = vec![0.0; 100];
    for i in 0..100 {
        buffer[i] = i as f64 / 100.0;
    }
    // Duplicate the buffer for secodn channel
    buffer.extend(buffer.clone());

    let written = task.write(
        Timeout::Seconds(10.0),
        DataFillMode::GroupByChannel,
        Some(100),
        &buffer[..],
    )?;

    assert_eq!(written, 100);

    task.stop()?;
    Ok(())
}
