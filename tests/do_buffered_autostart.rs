use anyhow::Result;
use daqmx::channels::DigitalChannel;
use daqmx::tasks::output::{OutputTask, WriteOptions};
use daqmx::tasks::{DigitalOutput, Task};
use daqmx::types::{ClockEdge::Rising, DataFillMode, SampleMode, Timeout};
use serial_test::serial;

#[test]
#[serial]
fn test_do_buffered_write_autostart_false_then_start() -> Result<()> {
    const SAMPLES: u32 = 10;

    let do1 = DigitalChannel::builder("DO1", "PCIe-6363_test/port0/line1")?.build()?;
    let mut do_task: Task<DigitalOutput> = Task::new("DO buffered autostart false")?;
    do_task.create_channel(do1)?;

    // Finite buffered DO task
    do_task.configure_sample_clock_timing(
        None,
        1000.0,
        Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )?;

    // Waveform: 101010...
    let mut do_buffer = [0u8; SAMPLES as usize];
    for i in 0..do_buffer.len() {
        do_buffer[i] = if i % 2 == 0 { 1 } else { 0 };
    }

    let written = do_task.write_with_options(
        Timeout::Seconds(5.0),
        DataFillMode::GroupByChannel,
        Some(SAMPLES),
        &do_buffer,
        WriteOptions::default().auto_start(false), // Preload only
    )?;
    assert_eq!(written, SAMPLES as i32);

    // Now explicitly start -> should output and finish
    do_task.start()?;
    do_task
        .wait_until_done(Timeout::Seconds(5.0))
        .expect("DO task did not complete in time");

    do_task.stop()?;
    Ok(())
}
