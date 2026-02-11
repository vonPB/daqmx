mod common;
use anyhow::Result;
use daqmx::channels::*;
use daqmx::tasks::*;
use daqmx::types::*;
use rand::Rng;
use serial_test::serial;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

const NUM_READER_THREADS: usize = 100;
const SAMPLES_PER_READ: usize = 50;
const SAMPLE_RATE: f64 = 20_000.0; // Hz
const ACTIVE_READ_DURATION_MS: u64 = 3000;
const MAX_READ_LOOPS_PER_THREAD: usize = 1_000_000;

// DAQ device and channel to use. Ensure this is valid in your NI MAX simulation or actual hardware.
const PHYSICAL_CHANNEL: &str = "PCIe-6363_test/ai0";

/// This test spawns a large number of threads that continuously read from a single DAQ task
/// using cloned task handles. After a period, the main thread stops the task.
/// The test verifies that all threads terminate gracefully without panics, and that the
/// stop command is effective.
#[serial]
#[test]
fn test_extreme_concurrent_reads_and_stop() -> Result<()> {
    if common::test_device_or_skip()?.is_none() {
        return Ok(());
    }
    println!(
        "Starting test_extreme_concurrent_reads_and_stop with {} reader threads.",
        NUM_READER_THREADS
    );

    let mut task: Task<AnalogInput> = Task::new("extreme_stress_task")?;
    let ch1 = VoltageChannel::builder("stress_channel", PHYSICAL_CHANNEL)?
        .max(10.0)
        .build()?;
    task.create_channel(ch1)?;

    task.configure_sample_clock_timing(
        None, // Use internal clock source
        SAMPLE_RATE,
        ClockEdge::Rising,
        SampleMode::ContinuousSamples,
        SAMPLE_RATE as u64,
    )?;

    task.set_read_auto_start(false)?; // this disallows the read function from automatically starting the task, after being stopped
    task.start()?;
    println!("DAQ task started.");

    let mut thread_handles = Vec::new();

    let total_successful_reads_atomic = Arc::new(AtomicUsize::new(0));

    for i in 0..NUM_READER_THREADS {
        // Clone task for the new thread, testing the safety and correctness of using cloned task handles concurrently
        let mut task_clone = task.clone();
        let total_reads_clone_atomic = Arc::clone(&total_successful_reads_atomic);

        let handle = std::thread::spawn(move || -> Result<usize> {
            let mut thread_buffer = vec![0.0f64; SAMPLES_PER_READ];
            let mut successful_reads_in_this_thread = 0;
            let mut rng = rand::rng();

            // Loop for a large number of iterations. The expectation is that the task.stop()
            // call from the main thread will cause task_clone.read() to return an error,
            for loop_idx in 0..MAX_READ_LOOPS_PER_THREAD {
                match task_clone.read(
                    Timeout::Seconds(0.5),
                    DataFillMode::GroupByChannel,
                    Some(SAMPLES_PER_READ as u32),
                    &mut thread_buffer,
                ) {
                    Ok(samples_read) => {
                        // Successfully read samples.
                        if samples_read as usize == SAMPLES_PER_READ {
                            successful_reads_in_this_thread += 1;
                            total_reads_clone_atomic.fetch_add(1, Ordering::Relaxed);
                        } else if samples_read == 0 {
                            // No samples read, could be task stopping or just no new data available yet.
                            // Yield to allow other threads/task to progress.
                            std::thread::sleep(Duration::from_micros(50));
                        }
                        if successful_reads_in_this_thread > 0
                            && successful_reads_in_this_thread % 200 == 0
                        {
                            println!(
                                "Thread {}: {} successful reads.",
                                i, successful_reads_in_this_thread
                            );
                        }
                    }
                    Err(e) => {
                        // An error occurred during read. This is the expected path for exiting
                        // the loop when the main task is stopped.
                        println!(
                            "Thread {} exiting read loop due to error: {:?}. Loop iteration: {}. Successful reads in thread: {}",
                            i, e, loop_idx, successful_reads_in_this_thread
                        );
                        return Ok(successful_reads_in_this_thread);
                    }
                }
                std::thread::sleep(Duration::from_micros(rng.random_range(10..50)));
            }

            println!(
                "Thread {} completed MAX_READ_LOOPS_PER_THREAD ({}) without being stopped. This is unexpected. Successful reads: {}",
                i, MAX_READ_LOOPS_PER_THREAD, successful_reads_in_this_thread
            );
            Err(anyhow::anyhow!(
                "Thread {} was not stopped as expected after {} loops.",
                i,
                MAX_READ_LOOPS_PER_THREAD
            ))
        });
        thread_handles.push(handle);
    }

    println!(
        "All {} reader threads spawned. Running for ~{}ms before sending stop.",
        NUM_READER_THREADS, ACTIVE_READ_DURATION_MS
    );

    // Let the card run for ACTIVE_READ_DURATION_MS
    std::thread::sleep(Duration::from_millis(ACTIVE_READ_DURATION_MS));

    println!("Sending stop command to DAQ task...");
    task.stop()?; // Stop the main task. This should cause ongoing/future reads in threads to fail.
    println!("Stop command sent.");

    // Wait for threads to complete
    let mut total_thread_successful_reads_collected = 0;
    for (idx, handle) in thread_handles.into_iter().enumerate() {
        match handle.join() {
            Ok(thread_result) => match thread_result {
                Ok(reads_in_thread) => {
                    println!(
                        "Thread {} finished gracefully with {} successful reads.",
                        idx, reads_in_thread
                    );
                    total_thread_successful_reads_collected += reads_in_thread;
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Thread {} returned an error from its execution: {}",
                        idx,
                        e
                    ));
                }
            },
            Err(panic_payload) => {
                return Err(anyhow::anyhow!(
                    "Thread {} panicked. Test failed. Panic payload: {:?}",
                    idx,
                    panic_payload
                ));
            }
        }
    }

    println!(
        "All reader threads joined. Total successful reads collected from thread results: {}.",
        total_thread_successful_reads_collected
    );
    println!(
        "Total successful reads via shared AtomicCounter: {}.",
        total_successful_reads_atomic.load(Ordering::Relaxed)
    );

    assert_eq!(
        total_thread_successful_reads_collected,
        total_successful_reads_atomic.load(Ordering::Relaxed),
        "Mismatch between reads collected from threads and atomic counter!"
    );
    assert_eq!(
        task.is_done()?,
        true,
        "Task should be marked as done after stopping."
    );

    println!("test_extreme_concurrent_reads_and_stop completed successfully.");
    Ok(())
}
