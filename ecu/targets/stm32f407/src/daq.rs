use hal::ecu_hal::ECUDAQFrame;

pub struct DAQHandler {
    first_buffer: [ECUDAQFrame; 10],
    second_buffer: [ECUDAQFrame; 10],
    second_buffer_selected: bool,
    counter: usize,
    current_values: ECUDAQFrame,
    min_values: Option<ECUDAQFrame>,
    max_values: Option<ECUDAQFrame>,
}

impl DAQHandler {
    pub const fn new() -> Self {
        Self {
            first_buffer: [ECUDAQFrame::default(); 10],
            second_buffer: [ECUDAQFrame::default(); 10],
            second_buffer_selected: false,
            counter: 0,
            current_values: ECUDAQFrame::default(),
            min_values: None,
            max_values: None,
        }
    }

    pub fn add_daq_frame(&mut self, daq_frame: ECUDAQFrame) -> bool {
        if self.second_buffer_selected {
            self.second_buffer[self.counter] = daq_frame;
        } else {
            self.first_buffer[self.counter] = daq_frame;
        }

        self.current_values = daq_frame;

        match self.min_values {
            Some(mut mins) => {
                for values in mins.sensor_values.iter_mut().zip(daq_frame.sensor_values.iter()) {
                    if *values.0 > *values.1 {
                        *values.0 = *values.1;
                    }
                }
            },
            None => self.min_values = Some(daq_frame),
        }

        match self.max_values {
            Some(mut maxs) => {
                for values in maxs.sensor_values.iter_mut().zip(daq_frame.sensor_values.iter()) {
                    if *values.0 < *values.1 {
                        *values.0 = *values.1;
                    }
                }
            },
            None => self.max_values = Some(daq_frame),
        }

        self.counter += 1;

        if self.counter >= 10 {
            self.second_buffer_selected = !self.second_buffer_selected;
            self.counter = 0;

            return true;
        }

        false
    }

    pub fn get_inactive_buffer<'a>(&'a self) -> &'a [ECUDAQFrame; 10] {
        if self.second_buffer_selected {
            &self.first_buffer
        } else {
            &self.second_buffer
        }
    }

    pub fn get_values(&mut self) -> (ECUDAQFrame, ECUDAQFrame, ECUDAQFrame) {
        let mins = match self.min_values {
            Some(mins) => mins,
            None => self.current_values,
        };

        let maxs = match self.max_values {
            Some(maxs) => maxs,
            None => self.current_values,
        };

        (self.current_values, mins, maxs)
    }

    pub fn reset_ranges(&mut self) {
        self.min_values = None;
        self.max_values = None;
    }
}
