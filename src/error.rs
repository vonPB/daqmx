//! https://github.com/WiresmithTech/daqmx-rs

use log::warn;
/// Error handling types and functions.
use thiserror::Error;

use crate::{daqmx, types::buffer_to_string};
use anyhow::Result as AnyResult;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DaqmxError {
    /// A DAQmx Generated Error. The i32 is the return code and the string is the extended description.
    #[error("DAQmx Generated Error: {1}")]
    DaqmxError(i32, String),
    #[error("String Value Not Valid for DAQmx API. Probably Contains Null")]
    CStringError(#[from] std::ffi::NulError),
    #[error("String value from DAQmx API does not contain valid Unicode (UTF8). This should not be possible and probably indicates corruption")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("String property length changed between reading the required length and reading the value. This is likely a race condition with another piece of code and a retry will probably correct this.")]
    StringPropertyLengthChanged,
    #[error("Value for given type ({0}) isn't a value that is expected: {1}")]
    UnexpectedValue(String, i32),
}

pub fn handle_error(return_code: i32) -> AnyResult<()> {
    match return_code {
        0 => Ok(()), // Do nothing if no error.
        i32::MIN..=-1 => {
            // Use extended info for errors.
            unsafe {
                let mut buffer = vec![0i8; 2048];
                daqmx::DAQmxGetExtendedErrorInfo(buffer.as_mut_ptr(), 2048);
                let message = buffer_to_string(buffer);
                Err(DaqmxError::DaqmxError(return_code, message).into())
            }
        }
        1..=i32::MAX => {
            // Use error string for warning. Just report to log.
            unsafe {
                let mut buffer = vec![0i8; 2048];
                daqmx::DAQmxGetErrorString(return_code, buffer.as_mut_ptr(), 2048);
                let message = buffer_to_string(buffer);
                warn!("DAQmx Warning: {}", message);
            }
            Ok(())
        }
    }
}

/// Checks the return code and either:
///
/// * Errors if it is an unexpected error.
/// * Returns `true` if there is a size error.
/// * Returns `false` if there is no error.
pub fn string_property_size_error(return_code: i32) -> AnyResult<bool> {
    const TRUNCATED_WARNING: i32 = daqmx::DAQmxWarningCAPIStringTruncatedToFitBuffer as i32;
    match return_code {
        daqmx::DAQmxErrorBufferTooSmallForString | TRUNCATED_WARNING => {
            // Wrong size, go again.
            Ok(true)
        }
        //Given we know this rante of codes provides an error, the map should never be called.
        //Just used to satisfy the type system.
        i32::MIN..=-1 => handle_error(return_code).map(|()| false),
        _ => Ok(false),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_string_property_size_error() {
        assert_eq!(string_property_size_error(0).unwrap(), false);
        assert_eq!(
            string_property_size_error(daqmx::DAQmxErrorBufferTooSmallForString).unwrap(),
            true
        );
        assert_eq!(
            string_property_size_error(daqmx::DAQmxWarningCAPIStringTruncatedToFitBuffer as i32)
                .unwrap(),
            true
        );

        match string_property_size_error(-1000) {
            Err(e) => {
                if let Some(DaqmxError::DaqmxError(code, _)) = e.downcast_ref::<DaqmxError>() {
                    assert_eq!(*code, -1000);
                } else {
                    panic!("Expected DaqmxError::DaqmxError");
                }
            }
            Ok(_) => panic!("Expected an error"),
        }
    }
}
