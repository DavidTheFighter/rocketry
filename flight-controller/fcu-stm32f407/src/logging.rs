use shared::{FlashDataLogger, fcu_hal::FcuSensorData};


pub const PAGE_SIZE: usize = 256;
pub type DataLoggerType<'a> = FlashDataLogger<'a, FcuSensorData, fn(&[u8; PAGE_SIZE]) -> (), PAGE_SIZE>;

pub fn full_page_callback(data: &[u8; PAGE_SIZE]) {
    // TODO
}

pub fn erase_flash_chip() {
    
}
