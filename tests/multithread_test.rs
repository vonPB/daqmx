//! Integration tests to confirm multithreaded behaviour.
//!
use anyhow::Result;
use daqmx::channels::*;
use daqmx::tasks::*;
use daqmx::types::*;
use serial_test::serial;

#[serial]
#[test]
fn test_move_to_thread() -> Result<()> {
    let mut task: Task<AnalogInput> = Task::new("scalar")?;
    let ch1 = VoltageChannel::builder("my name", "PCIe-6363_test/ai0")?.build()?;
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

    let join_handle = std::thread::spawn(move || {
        let res = task.read(
            Timeout::Seconds(1.0),
            DataFillMode::GroupByChannel,
            Some(100),
            &mut buffer[..],
        );

        assert!(res.is_ok());
    });

    join_handle
        .join()
        .map_err(|_| anyhow::anyhow!("Thread panicked"))?;
    Ok(())
}

#[serial]
#[test]
/// This test will move the read to another thread but set stop from this thread.
/// This is a fairly commmon case for multithreading a task.
fn test_control_from_thread() -> Result<()> {
    let mut task: Task<AnalogInput> = Task::new("scalar")?;
    let ch1 = VoltageChannel::builder("my name", "PCIe-6363_test/ai0")?.build()?;
    task.create_channel(ch1)?;
    task.configure_sample_clock_timing(
        None,
        1000.0,
        ClockEdge::Rising,
        SampleMode::ContinuousSamples,
        1000,
    )?;

    let mut buffer = [0.0; 1000];

    task.set_read_auto_start(false)?;
    task.start()?;

    assert_eq!(task.is_done()?, false);

    let mut thread_task = task.clone();

    let join_handle_1 = std::thread::spawn(move || {
        println!("Starting read 1");
        for _ in 0..10 {
            let result = thread_task.read(
                Timeout::Seconds(1.0),
                DataFillMode::GroupByChannel,
                Some(100),
                &mut buffer[..],
            );

            if result.is_err() {
                return;
            } else {
                println!("First: {:?}", result);
            }
        }

        //If we complete the iterations we weren't stopped. panic.
        panic!("Expected thread to be stopped by the task being stopped.");
    });

    let mut thread_task = task.clone();
    let join_handle_2 = std::thread::spawn(move || {
        println!("Starting read 2");
        for _ in 0..10 {
            let result = thread_task.read(
                Timeout::Seconds(1.0),
                DataFillMode::GroupByChannel,
                Some(100),
                &mut buffer[..],
            );

            if result.is_err() {
                return;
            } else {
                println!("Second: {:?}", result);
            }
        }

        //If we complete the iterations we weren't stopped. panic.
        panic!("Expected thread to be stopped by the task being stopped.");
    });

    std::thread::sleep(std::time::Duration::from_millis(500));
    println!("Sending stop");
    task.stop()?;

    join_handle_1
        .join()
        .map_err(|_| anyhow::anyhow!("Thread 1 panicked"))?;
    join_handle_2
        .join()
        .map_err(|_| anyhow::anyhow!("Thread 2 panicked"))?;
    Ok(())
}
