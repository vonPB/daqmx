mod common;
use anyhow::Result;
use daqmx::tasks::output::OutputTask;
use daqmx::tasks::AnalogInput;
use daqmx::tasks::AnalogOutput;
use daqmx::tasks::DigitalInput;
use daqmx::tasks::DigitalOutput;
use serial_test::serial;

use daqmx::channels::*;
use daqmx::tasks::InputTask;
use daqmx::tasks::Task;
use daqmx::types::ClockEdge::Rising;
use daqmx::types::DataFillMode;
use daqmx::types::SampleMode;
use daqmx::types::Timeout;

#[test]
#[serial]
fn test_ai_di() -> Result<()> {
    if common::test_device_or_skip()?.is_none() {
        return Ok(());
    }
    let ai1 = VoltageChannel::builder("my name", "PCIe-6363_test/ai1")?
        .max(1.0)
        .terminal_config(AnalogTerminalConfig::RSE)
        .build()?;

    let mut ai_task: Task<AnalogInput> = Task::new("")?;
    ai_task.create_channel(ai1)?;

    ai_task.configure_sample_clock_timing(
        None,
        10.0,
        Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;

    let di1 = DigitalChannel::builder("my_digital_input", "PCIe-6363_test/port0/line0")?.build()?;
    let mut di_task: Task<DigitalInput> = Task::new("DI")?;
    di_task.create_channel(di1)?;

    di_task.configure_sample_clock_timing(
        None,
        100.0,
        Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;

    ai_task.start()?;
    di_task.start()?;

    const SAMPLES: usize = 10;
    let mut ai_buffer = [0.0; SAMPLES];

    ai_task.read(
        Timeout::Seconds(1.2),
        DataFillMode::GroupByChannel,
        None,
        &mut ai_buffer,
    )?;

    let mut di_buffer = [0u8; SAMPLES];

    di_task.read(
        Timeout::Seconds(1.2),
        DataFillMode::GroupByChannel,
        None,
        &mut di_buffer,
    )?;

    ai_task.stop()?;
    di_task.stop()?;

    println!("AI buffer: {:?}", ai_buffer);
    println!("DI buffer: {:?}", di_buffer);
    Ok(())
}

// Test multiple task types (AI, AO, DI, DO) running simultaneously
#[test]
#[serial]
fn test_combined_tasks() -> Result<()> {
    if common::test_device_or_skip()?.is_none() {
        return Ok(());
    }
    const SAMPLES: usize = 10;

    // Analog Input
    let ai1 = VoltageChannel::builder("AI1", "PCIe-6363_test/ai1")?
        .max(1.0)
        .terminal_config(AnalogTerminalConfig::RSE)
        .build()?;

    let mut ai_task: Task<AnalogInput> = Task::new("AI Task")?;
    ai_task.create_channel(ai1)?;

    ai_task.configure_sample_clock_timing(
        None,
        10.0,
        Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;

    // Analog Output
    let ao1 = VoltageChannel::builder("AO1", "PCIe-6363_test/ao1")?
        .max(1.0)
        .build()?;

    let mut ao_task: Task<AnalogOutput> = Task::new("AO Task")?;
    ao_task.create_channel(ao1)?;

    ao_task.configure_sample_clock_timing(
        None,
        100.0,
        Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;

    // Digital Input
    let di1 = DigitalChannel::builder("DI1", "PCIe-6363_test/port0/line0")?.build()?;
    let mut di_task: Task<DigitalInput> = Task::new("DI Task")?;
    di_task.create_channel(di1)?;

    di_task.configure_sample_clock_timing(
        None,
        100.0,
        Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;

    // Digital Output
    let do1 = DigitalChannel::builder("DO1", "PCIe-6363_test/port0/line1")?.build()?;
    let mut do_task: Task<DigitalOutput> = Task::new("DO Task")?;
    do_task.create_channel(do1)?;

    // do_task.configure_sample_clock_timing(None, 500.0, Rising, SampleMode::ContinuousSamples, 0)?;

    // Start all tasks
    ai_task.start()?;
    ao_task.start()?;
    di_task.start()?;
    // do_task.start()?; // autostarts

    // Prepare buffers
    let mut ai_buffer = [0.0; SAMPLES];
    let mut ao_buffer = [0.0; SAMPLES];
    for i in 0..SAMPLES {
        ao_buffer[i] = i as f64 / SAMPLES as f64;
    }
    let mut di_buffer = [0u8; SAMPLES];
    let do_buffer = [1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8, 1u8, 0u8];

    // Write to Analog Output
    let written_ao = ao_task.write(
        Timeout::Seconds(10.0),
        DataFillMode::GroupByChannel,
        Some(SAMPLES as u32),
        &ao_buffer[..],
    )?;
    assert_eq!(written_ao, SAMPLES as i32);

    // Write to Digital Output
    let mut do_task_clone = do_task.clone();
    let handle = std::thread::spawn(move || {
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(10));

            let written_do = do_task_clone
                .write(
                    Timeout::Seconds(10.0),
                    DataFillMode::GroupByChannel,
                    Some(SAMPLES as u32),
                    &do_buffer,
                )
                .unwrap();

            assert_eq!(written_do, SAMPLES as i32);
        }
    });

    // Read from Analog Input
    ai_task.read(
        Timeout::Seconds(1.2),
        DataFillMode::GroupByChannel,
        None,
        &mut ai_buffer,
    )?;

    // Read from Digital Input
    di_task.read(
        Timeout::Seconds(1.2),
        DataFillMode::GroupByChannel,
        None,
        &mut di_buffer,
    )?;

    // Stop all tasks
    ai_task.stop()?;
    ao_task.stop()?;
    di_task.stop()?;
    do_task.stop()?;

    assert_eq!(handle.join().is_ok(), true);

    // Print buffers for verification
    println!("AI buffer: {:?}", ai_buffer);
    println!("AO buffer: {:?}", ao_buffer);
    println!("DI buffer: {:?}", di_buffer);
    println!("DO buffer: {:?}", do_buffer);

    Ok(())
}

#[test]
#[serial]
fn test_delayed_ai_read() -> Result<()> {
    if common::test_device_or_skip()?.is_none() {
        return Ok(());
    }
    const SAMPLES: usize = 600;

    let ai1 = VoltageChannel::builder("my name", "PCIe-6363_test/ai1")?
        .max(5.0)
        .terminal_config(AnalogTerminalConfig::RSE)
        .build()?;

    let mut ai_task: Task<AnalogInput> = Task::new("")?;
    ai_task.create_channel(ai1)?;

    ai_task.configure_sample_clock_timing(
        None,
        1000.0,
        Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;

    ai_task.start()?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    let mut ai_buffer = [0.0; SAMPLES];
    ai_task.read(
        Timeout::Seconds(0.2),
        DataFillMode::GroupByChannel,
        None,
        &mut ai_buffer,
    )?;

    assert_eq!(ai_buffer.len(), SAMPLES);
    assert_ne!(ai_buffer[0], 0.0);
    assert_ne!(ai_buffer[SAMPLES - 1], 0.0);

    ai_task.stop()?;
    Ok(())
}
