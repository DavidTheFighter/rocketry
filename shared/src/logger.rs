use core::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};

use postcard::{
    from_bytes_cobs,
    ser_flavors::{Cobs, Slice},
    serialize_with_flavor,
};

pub const SERIALIZE_BUFFER_SIZE: usize = 128;

pub trait DataPointLogger<T> {
    fn log_data_point(&mut self, data_point: &T);
    fn get_bytes_logged(&self) -> u32;
    fn set_logging_enabled(&mut self, enabled: bool);
}

pub struct FlashDataLogger<'a, T, F, const PAGE_SIZE: usize> {
    buffer0: &'a mut [u8; PAGE_SIZE],
    buffer1: &'a mut [u8; PAGE_SIZE],
    active_buffer: usize,
    active_buffer_index: usize,
    logging_enabled: bool,
    bytes_logged: u32,
    full_page_callback: Option<F>,
    _marker: PhantomData<T>,
}

impl<'a, T, F, const PAGE_SIZE: usize> DataPointLogger<T> for FlashDataLogger<'a, T, F, PAGE_SIZE>
where
    T: Serialize + DeserializeOwned,
    F: Fn(&[u8; PAGE_SIZE]),
{
    fn log_data_point(&mut self, data_point: &T) {
        if !self.logging_enabled {
            return;
        }

        let mut data_buffer = [0u8; SERIALIZE_BUFFER_SIZE];
        let serialized_size = Self::serialize_data_point(data_point, &mut data_buffer)
            .expect("Failed to serialize data point");

        self.bytes_logged += (serialized_size + 1) as u32;

        self.put_byte(serialized_size as u8);
        for byte in &data_buffer[0..serialized_size] {
            self.put_byte(*byte);
        }
    }

    fn get_bytes_logged(&self) -> u32 {
        self.bytes_logged
    }

    fn set_logging_enabled(&mut self, enabled: bool) {
        self.logging_enabled = enabled;
    }
}

impl<'a, T, F, const PAGE_SIZE: usize> FlashDataLogger<'a, T, F, PAGE_SIZE>
where
    T: Serialize + DeserializeOwned,
    F: Fn(&[u8; PAGE_SIZE]),
{
    pub fn new(
        buffer0: &'a mut [u8; PAGE_SIZE],
        buffer1: &'a mut [u8; PAGE_SIZE],
        full_page_callback: Option<F>,
    ) -> Self {
        Self {
            buffer0,
            buffer1,
            active_buffer: 0,
            active_buffer_index: 0,
            logging_enabled: false,
            bytes_logged: 0,
            full_page_callback,
            _marker: PhantomData {},
        }
    }

    pub fn retrieve_data_point(&self, buffer: &mut dyn Iterator<Item = &u8>) -> Option<T> {
        let size = (*buffer.next()?) as usize;
        let mut working_buffer = [0u8; SERIALIZE_BUFFER_SIZE];

        for byte in working_buffer.iter_mut().take(size) {
            *byte = *buffer.next()?;
        }

        Self::deserialize_data_point(&mut working_buffer[0..size])
    }

    pub fn active_buffer(&mut self) -> &mut [u8] {
        if self.active_buffer == 0 {
            &mut self.buffer0[0..self.active_buffer_index]
        } else {
            &mut self.buffer1[0..self.active_buffer_index]
        }
    }

    fn put_byte(&mut self, b: u8) {
        if self.active_buffer == 0 {
            self.buffer0[self.active_buffer_index] = b;
        } else {
            self.buffer1[self.active_buffer_index] = b;
        };

        self.active_buffer_index += 1;

        if self.active_buffer_index >= PAGE_SIZE {
            self.flip_buffer();

            if let Some(callback) = &self.full_page_callback {
                if self.active_buffer == 0 {
                    callback(self.buffer1);
                } else {
                    callback(self.buffer0);
                }
            }
        }
    }

    fn flip_buffer(&mut self) {
        self.active_buffer_index = 0;
        self.active_buffer = (self.active_buffer + 1) % 2;
    }

    fn serialize_data_point(data: &T, buffer: &mut [u8]) -> Option<usize> {
        match Cobs::try_new(Slice::new(buffer)) {
            Ok(flavor) => {
                let serialized = serialize_with_flavor::<T, Cobs<Slice>, &mut [u8]>(data, flavor);

                match serialized {
                    Ok(output_buffer) => Some(output_buffer.len()),
                    Err(_) => None,
                }
            }
            Err(_) => None,
        }
    }

    fn deserialize_data_point(buffer: &mut [u8]) -> Option<T> {
        from_bytes_cobs(buffer).ok()
    }
}

pub struct DataPointLoggerMock;

impl<T> DataPointLogger<T> for DataPointLoggerMock {
    fn log_data_point(&mut self, _data_point: &T) {}
    fn get_bytes_logged(&self) -> u32 { 0 }
    fn set_logging_enabled(&mut self, _enabled: bool) {}
}

#[cfg(test)]
pub mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum TestDataPoint {
        Data0 { data0: u8, data1: u8 },
        Data1 { data0: u16, data1: u8, data2: u8 },
        Data2 { data0: u32 },
    }

    #[test]
    fn test_serialize_deserialize() {
        const PAGE_SIZE: usize = 64;
        let mut buffer0 = [0_u8; PAGE_SIZE];
        let mut buffer1 = [0_u8; PAGE_SIZE];

        let data_points = [
            TestDataPoint::Data0 {
                data0: 42,
                data1: 37,
            },
            TestDataPoint::Data1 {
                data0: 310,
                data1: 0,
                data2: 255,
            },
            TestDataPoint::Data2 { data0: 0x12345678 },
            TestDataPoint::Data0 {
                data0: 17,
                data1: 84,
            },
            TestDataPoint::Data2 { data0: 0x87654321 },
        ];

        let full_page = |_buffer: &[u8; PAGE_SIZE]| {
            panic!("Buffer should not be full yet!");
        };

        let mut logger = FlashDataLogger::new(&mut buffer0, &mut buffer1, Some(full_page));
        logger.set_logging_enabled(true);

        for data_point in &data_points {
            logger.log_data_point(data_point);
        }

        let mut buffer_copy = [0_u8; PAGE_SIZE];
        for (src, dst) in buffer_copy
            .iter_mut()
            .zip(logger.active_buffer().into_iter())
        {
            *src = *dst;
        }

        let mut buffer_iter = buffer_copy.iter();
        for src_data_point in &data_points {
            let cmp_data_point = logger.retrieve_data_point(&mut buffer_iter);
            println!("Comparing {:?} and {:?}", src_data_point, cmp_data_point);
            assert_eq!(*src_data_point, cmp_data_point.unwrap());
        }
    }

    #[test]
    fn test_flip_buffer() {
        const PAGE_SIZE: usize = 64;
        let mut buffer0 = [0_u8; PAGE_SIZE];
        let mut buffer1 = [0_u8; PAGE_SIZE];

        let data_points = [
            TestDataPoint::Data0 {
                data0: 42,
                data1: 37,
            },
            TestDataPoint::Data0 {
                data0: 18,
                data1: 255,
            },
            TestDataPoint::Data1 {
                data0: 310,
                data1: 0,
                data2: 255,
            },
            TestDataPoint::Data2 { data0: 0x12345678 },
            TestDataPoint::Data0 {
                data0: 17,
                data1: 84,
            },
            TestDataPoint::Data2 { data0: 0x87654321 },
            TestDataPoint::Data2 { data0: 0x13243546 },
            TestDataPoint::Data2 { data0: 0x87654321 },
            TestDataPoint::Data2 { data0: 0x97864532 },
        ];

        let full_page = |_buffer: &[u8; PAGE_SIZE]| {};

        let mut logger = FlashDataLogger::new(&mut buffer0, &mut buffer1, Some(full_page));
        logger.set_logging_enabled(true);

        for data_point in &data_points {
            logger.log_data_point(data_point);
        }

        let mut buffer_copy = [0_u8; PAGE_SIZE];
        for (src, dst) in buffer_copy
            .iter_mut()
            .zip(logger.active_buffer().into_iter())
        {
            *src = *dst;
        }

        println!("{:?}", logger.active_buffer());
        println!("{:?}", buffer_copy);

        // Make a new temporary data logger so we can use the old buffers
        let mut tmp_buffer0 = [0_u8; PAGE_SIZE];
        let mut tmp_buffer1 = [0_u8; PAGE_SIZE];
        let logger = FlashDataLogger::new(&mut tmp_buffer0, &mut tmp_buffer1, Some(full_page));

        let mut buffer_iter = buffer0.iter().chain(buffer1.iter());
        for src_data_point in &data_points {
            let cmp_data_point = logger.retrieve_data_point(&mut buffer_iter);
            println!("Comparing {:?} and {:?}", src_data_point, cmp_data_point);
            assert_eq!(*src_data_point, cmp_data_point.unwrap());
        }
    }
}
