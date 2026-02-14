mod common;
use anyhow::Result;
use daqmx::channels::{AnalogTerminalConfig, DigitalChannel, VoltageChannel};
use daqmx::tasks::output::{OutputTask, WriteOptions};
use daqmx::tasks::{AnalogInput, DigitalOutput, Task};
use daqmx::types::{ClockEdge::Rising, DataFillMode, SampleMode, Timeout};
use serial_test::serial;

#[test]
#[serial]
fn test_ai_start_trigger_releases_do() -> Result<()> {
    if common::test_device_or_skip()?.is_none() {
        return Ok(());
    }
    const SAMPLES: u32 = 10;
    let dev = "PCIe-6363_test";

    // AI task (start trigger source)
    let ai1 = VoltageChannel::builder("AI1", format!("{dev}/ai1"))?
        .max(1.0)
        .terminal_config(AnalogTerminalConfig::RSE)
        .build()?;

    let mut ai_task: Task<AnalogInput> = Task::new("AI master")?;
    ai_task.create_channel(ai1)?;
    ai_task.configure_sample_clock_timing(
        None,
        1000.0,
        Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;

    // DO task (triggered)
    let do1 = DigitalChannel::builder("DO1", format!("{dev}/port0/line1"))?.build()?;
    let mut do_task: Task<DigitalOutput> = Task::new("DO slave")?;
    do_task.create_channel(do1)?;
    do_task.configure_sample_clock_timing(
        None,
        1000.0,
        Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;

    // Configure DO to wait for AI start trigger
    let ai_start = format!("/{dev}/ai/StartTrigger");
    do_task.configure_trigger(&ai_start, Rising)?;

    // Preload DO buffer WITHOUT starting
    let do_buffer: Vec<u8> = (0..SAMPLES)
        .map(|i| if i % 2 == 0 { 1 } else { 0 })
        .collect();
    let written = do_task.write_with_options(
        Timeout::Seconds(5.0),
        DataFillMode::GroupByChannel,
        Some(SAMPLES),
        &do_buffer,
        WriteOptions::default().auto_start(false),
    )?;
    assert_eq!(written, SAMPLES as i32);

    // Arm DO first (it should wait for trigger)
    do_task.start()?;

    // Start AI -> emits ai/StartTrigger -> releases DO
    ai_task.start()?;

    // Both should complete
    do_task.wait_until_done(Timeout::Seconds(5.0))?;
    ai_task.wait_until_done(Timeout::Seconds(5.0))?;

    do_task.stop()?;
    ai_task.stop()?;

    Ok(())
}
