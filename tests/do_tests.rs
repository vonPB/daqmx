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
    let ch1 = DigitalChannel::new("my_digital_input", "PCIe-6363_test/port0/line0")
        .unwrap()
        .build()
        .unwrap();

    let ch2 = DigitalChannel::new(
        "my_digital_input2",
        "PCIe-6363_test/port0/line1, PCIe-6363_test/port0/line2",
    )
    .unwrap()
    .build()
    .unwrap();

    let mut task: Task<DigitalOutput> = Task::new("").unwrap();
    task.create_channel(ch1).unwrap();
    task.create_channel(ch2).unwrap();

    // task.configure_sample_clock_timing(None, 100.0, Rising, SampleMode::FiniteSamples, 3)
    //     .unwrap();

    let configured: DigitalChannelBase<DigitalOutput> =
        task.get_channel("my_digital_input").unwrap();

    assert_eq!(
        configured.physical_channel().unwrap(),
        "PCIe-6363_test/port0/line0".to_owned()
    );

    task.set_read_auto_start(true).unwrap();

    let buffer = [
        1u8, 0u8, 1u8, // Channel 0: 3 samples
        1u8, 0u8, 1u8, // Channel 1: 3 samples
        1u8, 0u8, 1u8, // Channel 2: 3 samples
    ];

    println!("{} {}: {:?}", "🪚", "buffer source addr", buffer.as_ptr());

    let written = task
        .write(
            Timeout::Seconds(10.0),       // Timeout
            DataFillMode::GroupByChannel, // Data fill mode: Group by channel
            Some(3),                      // 3 samples per channel
            &buffer,                      // Data buffer
        )
        .unwrap();

    assert_eq!(written, 3);

    unsafe {
        let buffer = [
            1u8, 0u8, 1u8, // Channel 0: 3 samples
            1u8, 0u8, 1u8, // Channel 1: 3 samples
            1u8, 0u8, 1u8, // Channel 2: 3 samples
        ];

        let mut written = 0;
        handle_error(daqmx::daqmx::DAQmxWriteDigitalLines(
            task.raw_handle_unsafe(),
            3,
            daqmx::daqmx::bool32::from(true),
            10.0,
            // daqmx::daqmx::DAQmx_Val_GroupByChannel as u32,
            DataFillMode::GroupByChannel.into(),
            buffer.as_ptr(),
            &mut written,
            std::ptr::null_mut(),
        ))
        .unwrap();

        println!("{} {}: {:?}", "🪚", "buffer", buffer);
        println!("{} {}: {:?}", "🪚", "written", written);
    }

    // Stop the task
    task.stop().unwrap();
}

#[test]
#[serial]
fn test_manaul() -> anyhow::Result<()> {
    let mut task: Task<DigitalOutput> = Task::new("").unwrap();

    unsafe {
        let port = std::ffi::CString::new("PCIe-6363_test/port0/line0, PCIe-6363_test/port0/line1")
            .unwrap();

        handle_error(daqmx::daqmx::DAQmxCreateDOChan(
            task.raw_handle_unsafe(),
            port.as_ptr(),
            std::ptr::null(),
            daqmx::daqmx::DAQmx_Val_ChanForAllLines,
        ))?;
    }

    unsafe {
        let port = std::ffi::CString::new("PCIe-6363_test/port0/line2").unwrap();

        handle_error(daqmx::daqmx::DAQmxCreateDOChan(
            task.raw_handle_unsafe(),
            port.as_ptr(),
            std::ptr::null(),
            daqmx::daqmx::DAQmx_Val_ChanForAllLines,
        ))?;
    }

    task.start().unwrap();

    unsafe {
        let buffer = [
            1u8, 0u8, 1u8, // Channel 0: 3 samples
            1u8, 0u8, 1u8, // Channel 1: 3 samples
            1u8, 0u8, 1u8, // Channel 2: 3 samples
        ];

        let mut written = 0;
        handle_error(daqmx::daqmx::DAQmxWriteDigitalLines(
            task.raw_handle_unsafe(),
            3,
            daqmx::daqmx::bool32::from(true),
            10.0,
            daqmx::daqmx::DAQmx_Val_GroupByChannel as u32,
            buffer.as_ptr(),
            &mut written,
            std::ptr::null_mut(),
        ))?;

        println!("{} {}: {:?}", "🪚", "buffer", buffer);
        println!("{} {}: {:?}", "🪚", "written", written);
    }

    Ok(())
}
