use anyhow::Result;
use daqmx::channels::{CounterOutputPulseTimeChannel, DigitalChannel, VoltageChannel};
use daqmx::info;
use daqmx::tasks::output::{OutputTask, WriteOptions};
use daqmx::tasks::{
    AnalogInput, CounterOutput, CounterOutputTask, DigitalInput, DigitalOutput, Task,
};
use daqmx::types::{ClockEdge, DataFillMode, IdleState, SampleMode, Timeout};
use serial_test::serial;

fn test_device_or_skip() -> Result<Option<String>> {
    let dev = "PCIe-6363_test".to_string();
    let devices = info::get_device_names()?;

    if devices.iter().any(|d| d == &dev) {
        Ok(Some(dev))
    } else {
        eprintln!("Skipping test: required device '{}' not present", dev);
        Ok(None)
    }
}

#[test]
#[serial]
fn pfi_master_trigger_starts_ai_di_do() -> Result<()> {
    let Some(dev) = test_device_or_skip()? else {
        return Ok(());
    };

    const SAMPLES: u32 = 100;
    let rate = 10_000.0;

    let mut ai_task: Task<AnalogInput> = Task::new("sync-ai")?;
    let ai_ch = VoltageChannel::builder(format!("{dev}_ai0"), format!("{dev}/ai0"))?.build()?;
    ai_task.create_channel(ai_ch)?;
    ai_task.configure_sample_clock_timing(
        None,
        rate,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;
    ai_task.configure_trigger(&format!("/{dev}/PFI0"), ClockEdge::Rising)?;

    let mut di_task: Task<DigitalInput> = Task::new("sync-di")?;
    let di_ch =
        DigitalChannel::builder(format!("{dev}_di0"), format!("{dev}/port0/line0"))?.build()?;
    di_task.create_channel(di_ch)?;
    let ai_sample_clock = format!("/{dev}/ai/SampleClock");
    if di_task
        .configure_sample_clock_timing(
            Some(&ai_sample_clock),
            rate,
            ClockEdge::Rising,
            SampleMode::FiniteSamples,
            SAMPLES as u64,
        )
        .is_err()
    {
        di_task.configure_sample_clock_timing(
            None,
            rate,
            ClockEdge::Rising,
            SampleMode::FiniteSamples,
            SAMPLES as u64,
        )?;
    }
    di_task.configure_trigger(&format!("/{dev}/PFI0"), ClockEdge::Rising)?;

    let mut do_task: Task<DigitalOutput> = Task::new("sync-do")?;
    let do_ch =
        DigitalChannel::builder(format!("{dev}_do0"), format!("{dev}/port0/line1"))?.build()?;
    do_task.create_channel(do_ch)?;
    do_task.configure_sample_clock_timing(
        None,
        rate,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;
    do_task.configure_trigger(&format!("/{dev}/PFI0"), ClockEdge::Rising)?;

    let do_buffer: Vec<u8> = (0..SAMPLES)
        .map(|i| if i % 2 == 0 { 1 } else { 0 })
        .collect();
    let written = do_task.write_with_options(
        Timeout::Seconds(2.0),
        DataFillMode::GroupByChannel,
        Some(SAMPLES),
        &do_buffer,
        WriteOptions::default().auto_start(false),
    )?;
    assert_eq!(written, SAMPLES as i32);

    let mut co_task: Task<CounterOutput> = Task::new("sync-co")?;
    let co_ch =
        CounterOutputPulseTimeChannel::builder(format!("{dev}_co0"), format!("{dev}/ctr0"))?
            .idle_state(IdleState::Low)
            .low_time(0.000_005)
            .high_time(0.000_050)
            .build()?;
    co_task.create_channel(co_ch)?;
    co_task.configure_implicit_timing(SampleMode::FiniteSamples, 1)?;
    co_task.export_counter_output_event_to(&format!("/{dev}/PFI0"))?;

    // Arm AI/DI/DO first so they wait on the trigger edge.
    ai_task.start()?;
    di_task.start()?;
    do_task.start()?;

    // Fire one-shot pulse to release all triggered tasks.
    co_task.start_pulse()?;

    ai_task.wait_until_done(Timeout::Seconds(5.0))?;
    di_task.wait_until_done(Timeout::Seconds(5.0))?;
    do_task.wait_until_done(Timeout::Seconds(5.0))?;
    co_task.wait_until_done(Timeout::Seconds(5.0))?;

    Ok(())
}

#[test]
#[serial]
fn sensor_edge_triggers_another_shutter_group() -> Result<()> {
    let Some(dev) = test_device_or_skip()? else {
        return Ok(());
    };

    const SAMPLES: u32 = 50;

    let mut shutter_task: Task<DigitalOutput> = Task::new("shutter-group")?;
    let shutter_ch =
        DigitalChannel::builder(format!("{dev}_do2"), format!("{dev}/port0/line2"))?.build()?;
    shutter_task.create_channel(shutter_ch)?;
    shutter_task.configure_sample_clock_timing(
        None,
        1_000.0,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;
    shutter_task.configure_trigger(&format!("/{dev}/PFI1"), ClockEdge::Rising)?;

    let shutter_buf: Vec<u8> = (0..SAMPLES)
        .map(|i| if i > (SAMPLES / 4) { 1 } else { 0 })
        .collect();
    let written = shutter_task.write_with_options(
        Timeout::Seconds(2.0),
        DataFillMode::GroupByChannel,
        Some(SAMPLES),
        &shutter_buf,
        WriteOptions::default().auto_start(false),
    )?;
    assert_eq!(written, SAMPLES as i32);

    // Represent a sensor edge by generating a deterministic counter pulse and routing it to PFI1.
    let mut sensor_pulse: Task<CounterOutput> = Task::new("sensor-trigger")?;
    sensor_pulse.configure_one_shot_pulse_time(
        &format!("{dev}/ctr0"),
        0.000_005,
        0.000_100,
        IdleState::Low,
    )?;
    sensor_pulse.export_counter_output_event_to(&format!("/{dev}/PFI1"))?;

    shutter_task.start()?;
    sensor_pulse.start_pulse()?;
    shutter_task.wait_until_done(Timeout::Seconds(5.0))?;

    Ok(())
}
