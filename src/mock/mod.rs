use std::sync::{Arc, Mutex};

pub mod ai_mock;
pub mod ao_mock;
pub mod di_mock;
pub mod do_mock;

// Shared state for mock channels
/// Can be used instead of a real DAQmx task.
/// This might be useful for exploring algorithms with simulated hardware,
/// since simulated inputs are not related to the simulated outputs.
#[derive(Default)]
pub struct MockChannelState {
    pub analog_values: Mutex<Vec<f64>>,
    pub digital_values: Mutex<Vec<u32>>,
}

pub type SharedMockState = Arc<MockChannelState>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        mock::{
            ai_mock::MockAnalogInput, ao_mock::MockAnalogOutput, di_mock::MockDigitalInput,
            do_mock::MockDigitalOutput,
        },
        tasks::{output::OutputTask, InputTask},
        types::{DataFillMode, Timeout},
    };
    use std::sync::Arc;

    #[test]
    fn test_mock_analog_io() {
        let shared_state = Arc::new(MockChannelState::default());

        let mut analog_output = MockAnalogOutput::new(shared_state.clone());
        let mut analog_input = MockAnalogInput::new(shared_state.clone());

        // Write a value to the analog output
        analog_output
            .write_scalar(1.23, Timeout::Seconds(1.0))
            .unwrap();

        // Read the value from the analog input
        let value = analog_input.read_scalar(Timeout::Seconds(1.0)).unwrap();

        assert_eq!(value, 1.23);
    }

    #[test]
    fn test_mock_analog_io_buffer() {
        let shared_state = Arc::new(MockChannelState::default());

        let mut analog_output = MockAnalogOutput::new(shared_state.clone());
        let mut analog_input = MockAnalogInput::new(shared_state.clone());

        // Write buffer to the analog output
        let write_buffer = vec![1.0, 2.0, 3.0, 4.0];
        analog_output
            .write(
                Timeout::Seconds(1.0),
                DataFillMode::GroupByChannel,
                Some(write_buffer.len() as u32),
                &write_buffer,
            )
            .unwrap();

        // Read buffer from the analog input
        let mut read_buffer = vec![0.0; 4];
        let samples_read = analog_input
            .read(
                Timeout::Seconds(1.0),
                DataFillMode::GroupByChannel,
                Some(4),
                &mut read_buffer,
            )
            .unwrap();

        assert_eq!(samples_read, 4);
        assert_eq!(read_buffer, write_buffer);
    }

    #[test]
    fn test_mock_digital_io() {
        let shared_state = Arc::new(MockChannelState::default());

        let mut digital_output = MockDigitalOutput::new(shared_state.clone());
        let mut digital_input = MockDigitalInput::new(shared_state.clone());

        // Write a value to the digital output
        digital_output
            .write_scalar(1, Timeout::Seconds(1.0))
            .unwrap();

        // Read the value from the digital input
        let value = digital_input.read_scalar(Timeout::Seconds(1.0)).unwrap();

        assert_eq!(value, 1);
    }

    #[test]
    fn test_mock_digital_io_buffer() {
        let shared_state = Arc::new(MockChannelState::default());

        let mut digital_output = MockDigitalOutput::new(shared_state.clone());
        let mut digital_input = MockDigitalInput::new(shared_state.clone());

        // Write buffer to the digital output
        let write_buffer = vec![1, 0, 1, 1];
        digital_output
            .write(
                Timeout::Seconds(1.0),
                DataFillMode::GroupByChannel,
                Some(write_buffer.len() as u32),
                &write_buffer,
            )
            .unwrap();

        // Read buffer from the digital input
        let mut read_buffer = vec![0; 4];
        let samples_read = digital_input
            .read(
                Timeout::Seconds(1.0),
                DataFillMode::GroupByChannel,
                Some(4),
                &mut read_buffer,
            )
            .unwrap();

        assert_eq!(samples_read, 4);
        assert_eq!(read_buffer, write_buffer);
    }
}
