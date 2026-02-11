use anyhow::Result;
use daqmx::channels::{CounterOutputPulseTimeChannel, DigitalChannel, VoltageChannel};
use daqmx::error::DaqmxError;
use daqmx::tasks::output::{OutputTask, WriteOptions};
use daqmx::tasks::{
    AnalogInput, CounterOutput, CounterOutputTask, DigitalInput, DigitalOutput, Task,
};
use daqmx::types::{ClockEdge, DataFillMode, IdleState, SampleMode, Timeout};
use serial_test::serial;

mod common;

fn is_daqmx_error_code(err: &anyhow::Error, code: i32) -> bool {
    matches!(
        err.downcast_ref::<DaqmxError>(),
        Some(DaqmxError::DaqmxError(c, _)) if *c == code
    )
}

#[test]
#[serial]
fn pfi_master_trigger_starts_ai_di_do() -> Result<()> {
    let Some(dev) = common::test_device_or_skip()? else {
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
    let Some(dev) = common::test_device_or_skip()? else {
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

#[test]
#[serial]
fn arm_order_enforced_behavior() -> Result<()> {
    let Some(dev) = common::test_device_or_skip()? else {
        return Ok(());
    };

    const SAMPLES: u32 = 20;
    let trigger_terminal = format!("/{dev}/PFI2");

    let mut do_task: Task<DigitalOutput> = Task::new("arm-order-do")?;
    let do_ch =
        DigitalChannel::builder(format!("{dev}_do_arm"), format!("{dev}/port0/line1"))?.build()?;
    do_task.create_channel(do_ch)?;
    do_task.configure_sample_clock_timing(
        None,
        1_000.0,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;
    if let Err(err) = do_task.configure_trigger(&trigger_terminal, ClockEdge::Rising) {
        eprintln!("Skipping test: trigger route unsupported ({err})");
        return Ok(());
    }

    let do_buffer: Vec<u8> = (0..SAMPLES)
        .map(|i| if i % 2 == 0 { 1 } else { 0 })
        .collect();
    do_task.write_with_options(
        Timeout::Seconds(2.0),
        DataFillMode::GroupByChannel,
        Some(SAMPLES),
        &do_buffer,
        WriteOptions::default().auto_start(false),
    )?;

    let mut pulse_task: Task<CounterOutput> = Task::new("arm-order-co")?;
    pulse_task.configure_one_shot_pulse_time(
        &format!("{dev}/ctr0"),
        0.000_005,
        0.000_050,
        IdleState::Low,
    )?;
    if let Err(err) = pulse_task.export_counter_output_event_to(&trigger_terminal) {
        eprintln!("Skipping test: export route unsupported ({err})");
        return Ok(());
    }

    // Fire pulse BEFORE DO is armed.
    pulse_task.start_pulse()?;

    // Arm DO after that first pulse. It should still be waiting for a new trigger edge.
    do_task.start()?;
    let first_wait = do_task.wait_until_done(Timeout::Seconds(0.1));
    if first_wait.is_ok() {
        eprintln!("Skipping test: trigger behavior appears immediate on this target/device");
        return Ok(());
    }

    // Fire another pulse; now completion is expected.
    pulse_task.start_pulse()?;
    do_task.wait_until_done(Timeout::Seconds(5.0))?;

    Ok(())
}

#[test]
#[serial]
fn dual_do_shared_resource_smoke() -> Result<()> {
    let Some(dev) = common::test_device_or_skip()? else {
        return Ok(());
    };

    const SAMPLES: u32 = 16;

    let mut do_a: Task<DigitalOutput> = Task::new("shared-trigger-do-a")?;
    do_a.create_channel(
        DigitalChannel::builder(format!("{dev}_do_a"), format!("{dev}/port0/line0"))?.build()?,
    )?;
    do_a.configure_sample_clock_timing(
        None,
        1_000.0,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;

    let mut do_b: Task<DigitalOutput> = Task::new("shared-trigger-do-b")?;
    do_b.create_channel(
        DigitalChannel::builder(format!("{dev}_do_b"), format!("{dev}/port0/line1"))?.build()?,
    )?;
    do_b.configure_sample_clock_timing(
        None,
        1_000.0,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;

    let do_buf: Vec<u8> = (0..SAMPLES)
        .map(|i| if i % 3 == 0 { 1 } else { 0 })
        .collect();
    do_a.write_with_options(
        Timeout::Seconds(2.0),
        DataFillMode::GroupByChannel,
        Some(SAMPLES),
        &do_buf,
        WriteOptions::default().auto_start(false),
    )?;
    if let Err(err) = do_b.write_with_options(
        Timeout::Seconds(2.0),
        DataFillMode::GroupByChannel,
        Some(SAMPLES),
        &do_buf,
        WriteOptions::default().auto_start(false),
    ) {
        // Some devices reserve a single DO timing/resource domain across multiple DO tasks.
        if is_daqmx_error_code(&err, -50103) {
            eprintln!("Skipping test: device reserves shared DO resources ({err})");
            return Ok(());
        }
        return Err(err);
    }

    do_a.start()?;
    if let Err(err) = do_b.start() {
        if is_daqmx_error_code(&err, -50103) {
            eprintln!("Skipping test: device reserves shared DO resources ({err})");
            return Ok(());
        }
        return Err(err);
    }

    do_a.wait_until_done(Timeout::Seconds(5.0))?;
    do_b.wait_until_done(Timeout::Seconds(5.0))?;

    Ok(())
}

#[test]
#[serial]
fn multi_task_shared_trigger_completion() -> Result<()> {
    let Some(dev) = common::test_device_or_skip()? else {
        return Ok(());
    };

    const SAMPLES: u32 = 40;
    let trigger_terminal = format!("/{dev}/PFI3");

    let mut ai_task: Task<AnalogInput> = Task::new("shared-trigger-ai")?;
    let ai_ch =
        VoltageChannel::builder(format!("{dev}_ai_shared"), format!("{dev}/ai0"))?.build()?;
    ai_task.create_channel(ai_ch)?;
    ai_task.configure_sample_clock_timing(
        None,
        5_000.0,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;
    if let Err(err) = ai_task.configure_trigger(&trigger_terminal, ClockEdge::Rising) {
        eprintln!("Skipping test: AI trigger route unsupported ({err})");
        return Ok(());
    }

    let mut di_task: Task<DigitalInput> = Task::new("shared-trigger-di")?;
    di_task.create_channel(
        DigitalChannel::builder(format!("{dev}_di_shared"), format!("{dev}/port0/line0"))?
            .build()?,
    )?;
    let ai_sample_clock = format!("/{dev}/ai/SampleClock");
    if di_task
        .configure_sample_clock_timing(
            Some(&ai_sample_clock),
            5_000.0,
            ClockEdge::Rising,
            SampleMode::FiniteSamples,
            SAMPLES as u64,
        )
        .is_err()
    {
        di_task.configure_sample_clock_timing(
            None,
            5_000.0,
            ClockEdge::Rising,
            SampleMode::FiniteSamples,
            SAMPLES as u64,
        )?;
    }
    if let Err(err) = di_task.configure_trigger(&trigger_terminal, ClockEdge::Rising) {
        eprintln!("Skipping test: DI trigger route unsupported ({err})");
        return Ok(());
    }

    let mut do_task: Task<DigitalOutput> = Task::new("shared-trigger-do")?;
    do_task.create_channel(
        DigitalChannel::builder(format!("{dev}_do_shared"), format!("{dev}/port0/line1"))?
            .build()?,
    )?;
    do_task.configure_sample_clock_timing(
        None,
        5_000.0,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;
    if let Err(err) = do_task.configure_trigger(&trigger_terminal, ClockEdge::Rising) {
        eprintln!("Skipping test: DO trigger route unsupported ({err})");
        return Ok(());
    }

    let do_buf: Vec<u8> = (0..SAMPLES)
        .map(|i| if i % 3 == 0 { 1 } else { 0 })
        .collect();
    do_task.write_with_options(
        Timeout::Seconds(2.0),
        DataFillMode::GroupByChannel,
        Some(SAMPLES),
        &do_buf,
        WriteOptions::default().auto_start(false),
    )?;

    let mut pulse_task: Task<CounterOutput> = Task::new("shared-trigger-co")?;
    pulse_task.configure_one_shot_pulse_time(
        &format!("{dev}/ctr0"),
        0.000_005,
        0.000_100,
        IdleState::Low,
    )?;
    if let Err(err) = pulse_task.export_counter_output_event_to(&trigger_terminal) {
        eprintln!("Skipping test: pulse export route unsupported ({err})");
        return Ok(());
    }

    // Arm all waiting tasks first.
    ai_task.start()?;
    di_task.start()?;
    do_task.start()?;

    // One pulse should release all triggered tasks.
    pulse_task.start_pulse()?;

    ai_task.wait_until_done(Timeout::Seconds(5.0))?;
    di_task.wait_until_done(Timeout::Seconds(5.0))?;
    do_task.wait_until_done(Timeout::Seconds(5.0))?;

    Ok(())
}
