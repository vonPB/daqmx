use daqmx::channels::*;
use daqmx::tasks::DigitalInput;
use daqmx::tasks::InputTask;
use daqmx::tasks::Task;
use daqmx::types::ClockEdge::Rising;
use daqmx::types::DataFillMode;
use daqmx::types::SampleMode;
use daqmx::types::Timeout;
use serial_test::serial;

#[test]
#[serial]
fn test_digital_input_builder() {
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

    let mut task: Task<DigitalInput> = Task::new("").unwrap();
    task.create_channel(ch1).unwrap();
    task.create_channel(ch2).unwrap();

    task.configure_sample_clock_timing(None, 100.0, Rising, SampleMode::FiniteSamples, 10 as u64)
        .unwrap();

    let configured: DigitalChannelBase<DigitalInput> =
        task.get_channel("my_digital_input").unwrap();

    assert_eq!(
        configured.physical_channel().unwrap(),
        "PCIe-6363_test/port0/line0".to_owned()
    );

    task.start().unwrap();
    let mut buffer = [0u8; 40];

    task.read(
        Timeout::Seconds(10.0),
        DataFillMode::GroupByChannel,
        Some(10),
        &mut buffer,
    )
    .unwrap();

    println!("buffer: {:?}", buffer);

    task.stop().unwrap();
}
