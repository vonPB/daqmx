use anyhow::Result;
use daqmx::channels::DigitalChannel;
use daqmx::channels::DigitalChannelBase;
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
fn test_digital_output_builder() -> Result<()> {
    for _ in 0..100 {
        let ch1 =
            DigitalChannel::builder("my_digital_input", "PCIe-6363_test/port0/line0")?.build()?;

        let ch2 =
            DigitalChannel::builder("my_digital_input2", "PCIe-6363_test/port0/line1")?.build()?;

        let mut task: Task<DigitalOutput> = Task::new("")?;
        task.create_channel(ch1)?;
        task.create_channel(ch2)?;

        task.configure_sample_clock_timing(None, 100.0, Rising, SampleMode::FiniteSamples, 3)?;

        let configured: DigitalChannelBase<DigitalOutput> = task.get_channel("my_digital_input")?;

        assert_eq!(
            configured.physical_channel()?,
            "PCIe-6363_test/port0/line0".to_owned()
        );

        assert_eq!(task.channel_count, 2);

        task.set_read_auto_start(true)?;

        let buffer = [
            1u8, 0u8, 1u8, // Channel 0: 3 samples
            1u8, 0u8, 1u8, // Channel 1: 3 samples
        ];

        let written = task.write(
            Timeout::Seconds(10.0),       // Timeout
            DataFillMode::GroupByChannel, // Data fill mode: Group by channel
            Some(3),                      // 3 samples per channel
            &buffer,                      // Data buffer
        )?;

        assert_eq!(written, 3);

        // Stop the task
        task.stop()?;
    }
    Ok(())
}

#[test]
#[serial]
fn test_digital_output_builder_bool() -> Result<()> {
    for _ in 0..100 {
        let ch1 =
            DigitalChannel::builder("my_digital_input", "PCIe-6363_test/port0/line0")?.build()?;

        let ch2 =
            DigitalChannel::builder("my_digital_input2", "PCIe-6363_test/port0/line1")?.build()?;

        let mut task: Task<DigitalOutput> = Task::new("")?;
        task.create_channel(ch1)?;
        task.create_channel(ch2)?;

        task.configure_sample_clock_timing(None, 100.0, Rising, SampleMode::FiniteSamples, 3)?;

        let configured: DigitalChannelBase<DigitalOutput> = task.get_channel("my_digital_input")?;

        assert_eq!(
            configured.physical_channel()?,
            "PCIe-6363_test/port0/line0".to_owned()
        );

        assert_eq!(task.channel_count, 2);

        task.set_read_auto_start(true)?;

        let buffer = [
            true, false, true, // Channel 0: 3 samples
            true, false, true, // Channel 1: 3 samples
        ];

        let written = task.write(
            Timeout::Seconds(10.0),       // Timeout
            DataFillMode::GroupByChannel, // Data fill mode: Group by channel
            Some(3),                      // 3 samples per channel
            &buffer,                      // Data buffer
        )?;

        assert_eq!(written, 3);

        // Stop the task
        task.stop()?;
    }
    Ok(())
}
