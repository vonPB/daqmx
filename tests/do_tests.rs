use std::sync::Arc;

use daqmx::channels::DigitalChannel;
use daqmx::channels::DigitalChannelBase;
use daqmx::error::handle_error;
use daqmx::tasks::output::OutputTask;
use daqmx::tasks::DigitalOutput;
use daqmx::tasks::Task;
use daqmx::types::ClockEdge::Rising;
use daqmx::types::DataFillMode;
use daqmx::types::SampleMode;
use daqmx::types::Timeout;
use serial_test::serial;

#[test]
#[serial]
fn test_digital_output_builder() {
    for _ in 0..100 {
        let ch1 = DigitalChannel::new("my_digital_input", "PCIe-6363_test/port0/line0")
            .unwrap()
            .build()
            .unwrap();

        let ch2 = DigitalChannel::new("my_digital_input2", "PCIe-6363_test/port0/line1")
            .unwrap()
            .build()
            .unwrap();

        let mut task: Task<DigitalOutput> = Task::new("").unwrap();
        task.create_channel(ch1).unwrap();
        task.create_channel(ch2).unwrap();

        task.configure_sample_clock_timing(None, 100.0, Rising, SampleMode::FiniteSamples, 3)
            .unwrap();

        let configured: DigitalChannelBase<DigitalOutput> =
            task.get_channel("my_digital_input").unwrap();

        assert_eq!(
            configured.physical_channel().unwrap(),
            "PCIe-6363_test/port0/line0".to_owned()
        );

        assert_eq!(task.channel_count, 2);

        task.set_read_auto_start(true).unwrap();

        let buffer = [
            1u8, 0u8, 1u8, // Channel 0: 3 samples
            1u8, 0u8, 1u8, // Channel 1: 3 samples
        ];

        let written = task
            .write(
                Timeout::Seconds(10.0),       // Timeout
                DataFillMode::GroupByChannel, // Data fill mode: Group by channel
                Some(3),                      // 3 samples per channel
                &buffer,                      // Data buffer
            )
            .unwrap();

        assert_eq!(written, 3);

        // Stop the task
        task.stop().unwrap();
    }
}

#[test]
#[serial]
fn test_manual() -> anyhow::Result<()> {
    for _ in 0..100 {
        let mut task: Task<DigitalOutput> = Task::new("").unwrap();

        unsafe {
            let port =
                std::ffi::CString::new("PCIe-6363_test/port0/line0, PCIe-6363_test/port0/line1")
                    .unwrap();

            handle_error(daqmx::daqmx::DAQmxCreateDOChan(
                task.raw_handle_unsafe(),
                port.as_ptr(),
                std::ptr::null(),
                daqmx::daqmx::DAQmx_Val_ChanForAllLines,
            ))?;
        }

        unsafe {
            let port =
                std::ffi::CString::new("PCIe-6363_test/port0/line2, PCIe-6363_test/port0/line3")
                    .unwrap();

            handle_error(daqmx::daqmx::DAQmxCreateDOChan(
                task.raw_handle_unsafe(),
                port.as_ptr(),
                std::ptr::null(),
                daqmx::daqmx::DAQmx_Val_ChanForAllLines,
            ))?;
        }

        task.start().unwrap();

        let buffer = [
            1u8, 0u8, 1u8, // Channel 0: 3 samples
            1u8, 0u8, 1u8, // Channel 1: 3 samples
            1u8, 0u8, 1u8, // Channel 2: 3 samples
            1u8, 0u8, 1u8, // Channel 3: 3 samples
        ];

        let mut written = 0;

        handle_error(unsafe {
            daqmx::daqmx::DAQmxWriteDigitalLines(
                task.raw_handle_unsafe(),
                3,
                daqmx::daqmx::bool32::from(true),
                10.0,
                daqmx::daqmx::DAQmx_Val_GroupByChannel as u32,
                buffer.as_ptr(),
                &mut written,
                std::ptr::null_mut(),
            )
        })?;
    }
    Ok(())
}
