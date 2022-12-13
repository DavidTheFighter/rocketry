
#[derive(Copy, Clone)]
pub struct DAQFrame {
    pub adc1_in3: u16,
    pub adc1_in4: u16,
    pub adc2_in5: u16,
    pub adc2_in6: u16,
    pub adc3_in7: u16,
    pub adc3_in8: u16,
}

impl DAQFrame {
    pub const fn default() -> Self {
        Self {
            adc1_in3: 0,
            adc1_in4: 0,
            adc2_in5: 0,
            adc2_in6: 0,
            adc3_in7: 0,
            adc3_in8: 0,
        }
    }
}

pub struct DAQHandler {
    first_buffer: [DAQFrame; 10],
    second_buffer: [DAQFrame; 10],
    second_buffer_selected: bool,
    counter: usize,
}

impl DAQHandler {
    pub const fn new() -> Self {
        Self { 
            first_buffer: [DAQFrame::default(); 10],
            second_buffer: [DAQFrame::default(); 10],
            second_buffer_selected: false,
            counter: 0,
        }
    }

    pub fn add_daq_frame(&mut self, daq_frame: DAQFrame) -> bool {
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

    pub fn get_inactive_buffer<'a>(&'a self) -> &'a [u8] {
        let buffer = if self.second_buffer_selected {
            &self.first_buffer
        } else {
            &self.second_buffer
        };

        let (prefix, data, suffix) = unsafe { buffer.align_to::<u8>() };
            assert!(prefix.is_empty() && suffix.is_empty() &&
                    core::mem::align_of::<u8>() <= core::mem::align_of::<DAQFrame>(),
                    "Expected u8 alignment to be no stricter than DAQFrame alignment");

        data
    }
}