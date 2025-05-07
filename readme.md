# DAQmx Rust 

This is a Rust wrapper for the NI DAQmx library.
It aims to provide a safe and idiomatic API for the NI DAQmx library.

## Status

The following tasks are implemented:

* Analog Input
* Analog Output
* Digital Input
* Digital Output

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

## Credits

This project includes code and architectural inspiration from the [daqmx-rs](https://github.com/WiresmithTech/daqmx-rs) project by [WiresmithTech](https://github.com/WiresmithTech). The original project is licensed under the MIT license.

## License

This project is licensed under the MIT license. See the [LICENSE](LICENSE) file for more information.
