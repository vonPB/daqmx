use daqmx::tasks::AnalogInput;
use serial_test::serial;
use std::ffi::CString;

use daqmx::channels::*;
use daqmx::scales::LinearScale;
use daqmx::scales::PreScaledUnits;
use daqmx::tasks::InputTask;
use daqmx::tasks::Task;
use daqmx::types::ClockEdge;
use daqmx::types::ClockEdge::Rising;
use daqmx::types::DataFillMode;
use daqmx::types::SampleMode;
use daqmx::types::Timeout;

#[test]
#[serial]
fn test_voltage_input_builder() {
    let ch1 = VoltageChannel::new("my name", "PCIe-6363_test/ai1")
        .unwrap()
        .max(1.0)
        .min(-1.0)
        .terminal_config(AnalogTerminalConfig::RSE)
        .build()
        .unwrap();

    let mut task: Task<AnalogInput> = Task::new("").unwrap();
    task.create_channel(ch1).unwrap();

    let configured: VoltageInputChannel = task.get_channel("my name").unwrap();

    assert_eq!(
        configured.physical_channel().unwrap(),
        "PCIe-6363_test/ai1".to_owned()
    );
    assert_eq!(configured.ai_max().unwrap(), 1.0);
    assert_eq!(configured.ai_min().unwrap(), -1.0);
    assert_eq!(
        configured.ai_terminal_config().unwrap(),
        AnalogTerminalConfig::RSE
    );
    assert_eq!(configured.scale().unwrap(), VoltageScale::Volts);

    task.start().unwrap();

    const SAMPLES: usize = 10;
    let mut buffer = [0.0; SAMPLES];

    let scalar = task.read_scalar(Timeout::Seconds(10.0)).unwrap();
    assert_ne!(scalar, 0.0);
    task.stop().unwrap();

    task.configure_sample_clock_timing(
        None,
        100.0,
        Rising,
        SampleMode::FiniteSamples,
        SAMPLES as u64,
    )
    .unwrap();
    task.start().unwrap();

    task.read(
        Timeout::Seconds(1.0),
        DataFillMode::GroupByChannel,
        None,
        &mut buffer,
    )
    .unwrap();

    task.stop().unwrap();
}

#[test]
#[serial]
fn test_scalar_read() {
    let mut task: Task<AnalogInput> = Task::new("scalar").unwrap();
    let ch1 = VoltageChannel::new("my name", "PCIe-6363_test/ai0")
        .unwrap()
        .build()
        .unwrap();
    task.create_channel(ch1).unwrap();
    let res = task.read_scalar(Timeout::Seconds(1.0)).unwrap();
    assert_ne!(res, 0.0);
    drop(task);
}

#[test]
#[serial]
fn test_buffered_read() {
    let mut task: Task<AnalogInput> = Task::new("scalar").unwrap();
    let ch1 = VoltageChannel::new("my_name", "PCIe-6363_test/ai0")
        .unwrap()
        .build()
        .unwrap();
    task.create_channel(ch1).unwrap();
    task.configure_sample_clock_timing(
        None,
        1000.0,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        100,
    )
    .unwrap();

    let mut buffer = [0.0; 100];

    task.start().unwrap();
    task.read(
        Timeout::Seconds(1.0),
        DataFillMode::GroupByChannel,
        Some(100),
        &mut buffer[..],
    )
    .unwrap();
}

#[test]
#[serial]
fn test_stop() {
    let mut task: Task<AnalogInput> = Task::new("scalar").unwrap();
    let ch1 = VoltageChannel::new("my_name", "PCIe-6363_test/ai0")
        .unwrap()
        .build()
        .unwrap();
    task.create_channel(ch1).unwrap();
    task.configure_sample_clock_timing(
        None,
        1000.0,
        ClockEdge::Rising,
        SampleMode::FiniteSamples,
        100,
    )
    .unwrap();

    let mut buffer = [0.0; 100];

    task.set_read_auto_start(false).unwrap();
    task.start().unwrap();
    task.read(
        Timeout::Seconds(1.0),
        DataFillMode::GroupByChannel,
        Some(100),
        &mut buffer[..],
    )
    .unwrap();

    //now stop and confirm read response.
    task.stop().unwrap();
    let read_result = task.read(
        Timeout::Seconds(1.0),
        DataFillMode::GroupByChannel,
        Some(100),
        &mut buffer[..],
    );

    assert_eq!(read_result.is_err(), true);

    if let Some(daqmx::error::DaqmxError::DaqmxError(code, _)) = read_result
        .unwrap_err()
        .downcast_ref::<daqmx::error::DaqmxError>(
    ) {
        assert_eq!(*code, -200473);
    } else {
        panic!("Expected DaqmxError with code -200473");
    }
}

#[test]
#[serial]
fn test_voltage_input_builder_custom_scale() {
    // create custom scale first.
    let _scale = LinearScale::new("TestScale", 1.0, 1.5, PreScaledUnits::Volts, "test").unwrap();
    let ch1 = VoltageChannel::new("my name", "PCIe-6363_test/ai1")
        .unwrap()
        .scale(VoltageScale::CustomScale(Some(
            CString::new("TestScale").expect("Name Error"),
        )))
        .max(5.0)
        .min(-4.0)
        .terminal_config(AnalogTerminalConfig::RSE)
        .build()
        .unwrap();

    let mut task: Task<AnalogInput> = Task::new("custom scale").unwrap();
    task.create_channel(ch1).unwrap();

    let configured: VoltageInputChannel = task.get_channel("my name").unwrap();

    assert_eq!(
        configured.scale().unwrap(),
        VoltageScale::CustomScale(Some(CString::new("TestScale").expect("Name Error")))
    );

    task.start().unwrap();

    let mut buffer = [0.0; 10];
    task.read(
        Timeout::Seconds(1.0),
        DataFillMode::GroupByChannel,
        Some(10),
        &mut buffer,
    )
    .unwrap();

    assert_ne!(buffer[0], 0.0);
}
