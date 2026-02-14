# DAQmx Rust 

This is a Rust wrapper for the NI DAQmx library.
It aims to provide a safe and idiomatic API for the NI DAQmx library.

## Status

The following tasks are implemented:

* Analog Input
* Analog Output
* Digital Input
* Digital Output
* Counter Input
* Counter Output

each can be created and configured, and then read and written to.

The crate is used in an internal project, for a measurement machine that has been running 24/7 for over a year.
The implemented functionality is sufficient for its use case but might be expanded in the future.

## Dependencies

This project requires the National Instruments NI-DAQmx driver to be installed on the system.
[NI-DAQmx](https://www.ni.com/en-us/support/downloads/drivers/download.ni-daqmx.html) is a commercial product.

The crate is tested on Windows 11. If you are using a different operating system, you may need to modify the build script.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
daqmx = { git = "https://github.com/vonpb/daqmx" }
```

And this to your crate root:

```rust
use daqmx::channels::*;
use daqmx::tasks::*;
use daqmx::types::Timeout;
use anyhow::Result;

fn main() -> Result<()> {
    let mut task: Task<AnalogInput> = Task::new("scalar")?;
    let ch1 = VoltageChannel::builder("my name", "PCIe-6363_test/ai0")?.build()?;
    task.create_channel(ch1)?;

    let res = task.read_scalar(Timeout::Seconds(1.0))?;
    assert_ne!(res, 0.0);

    Ok(())
}
```

See the [tests](tests) directory for more examples.

## Deterministic Start Trigger

Use a one-shot hardware counter pulse to trigger multiple tasks deterministically.

```rust
use anyhow::Result;
use daqmx::channels::{DigitalChannel, VoltageChannel};
use daqmx::tasks::output::{OutputTask, WriteOptions};
use daqmx::tasks::{CounterOutputTask, AnalogInput, CounterOutput, DigitalOutput, Task};
use daqmx::types::{ClockEdge, DataFillMode, IdleState, SampleMode, Timeout};

fn deterministic_trigger(dev: &str) -> Result<()> {
    let trigger = format!("/{dev}/PFI0");

    let mut ai: Task<AnalogInput> = Task::new("ai")?;
    ai.create_channel(VoltageChannel::builder("ai0", format!("{dev}/ai0"))?.build()?)?;
    ai.configure_sample_clock_timing(None, 10_000.0, ClockEdge::Rising, SampleMode::FiniteSamples, 100)?;
    ai.configure_trigger(&trigger, ClockEdge::Rising)?;

    let mut do_task: Task<DigitalOutput> = Task::new("do")?;
    do_task.create_channel(DigitalChannel::builder("do0", format!("{dev}/port0/line0"))?.build()?)?;
    do_task.configure_sample_clock_timing(None, 10_000.0, ClockEdge::Rising, SampleMode::FiniteSamples, 100)?;
    do_task.configure_trigger(&trigger, ClockEdge::Rising)?;
    do_task.write_with_options(
        Timeout::Seconds(2.0),
        DataFillMode::GroupByChannel,
        Some(100),
        &vec![1u8; 100],
        WriteOptions::default().auto_start(false),
    )?;

    let mut co: Task<CounterOutput> = Task::new("co")?;
    co.configure_one_shot_pulse_time(&format!("{dev}/ctr0"), 5e-6, 50e-6, IdleState::Low)?;
    co.export_counter_output_event_to(&trigger)?;

    ai.start()?;
    do_task.start()?;
    co.start_pulse()?;
    Ok(())
}
```

Typical trigger source strings:
- `/DevX/PFI0`
- `/DevX/ai/StartTrigger`
- `/DevX/ai/SampleClock`

## Credits

This project includes code and architectural inspiration from the [daqmx-rs](https://github.com/WiresmithTech/daqmx-rs) project by [WiresmithTech](https://github.com/WiresmithTech). The original project is licensed under the MIT license.

## License

This project is licensed under the MIT license. See the [LICENSE](LICENSE) file for more information.
