
use hal::ecu_hal::ECUDAQFrame;

pub struct DAQHandler {
    first_buffer: [ECUDAQFrame; 10],
    second_buffer: [ECUDAQFrame; 10],
    second_buffer_selected: bool,
    counter: usize,
}

impl DAQHandler {
    pub const fn new() -> Self {
        Self { 
            first_buffer: [ECUDAQFrame::default(); 10],
            second_buffer: [ECUDAQFrame::default(); 10],
            second_buffer_selected: false,
            counter: 0,
        }
    }

    pub fn add_daq_frame(&mut self, daq_frame: ECUDAQFrame) -> bool {
        if self.second_buffer_selected {
            self.second_buffer[self.counter] = daq_frame;
        } else {
            self.first_buffer[self.counter] = daq_frame;
        }

        self.counter += 1;

        if self.counter >= 10 {
            self.second_buffer_selected = !self.second_buffer_selected;
            self.counter = 0;

            return true
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
}