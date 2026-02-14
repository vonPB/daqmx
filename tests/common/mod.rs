use anyhow::Result;
use daqmx::info;

pub fn test_device_or_skip() -> Result<Option<String>> {
    let dev = "PCIe-6363_test".to_string();
    let devices = info::get_device_names()?;

    if devices.iter().any(|d| d == &dev) {
        Ok(Some(dev))
    } else {
        eprintln!("Skipping test: required device '{}' not present", dev);
        Ok(None)
    }
}
