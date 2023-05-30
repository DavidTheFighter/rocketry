use core::fmt::Write;

use cortex_m::prelude::_embedded_hal_serial_Read;
use rtic::Mutex;
use hal::{fcu_log::{DataPoint, DataLogBuffer}, comms_hal::{Packet, NetworkAddress}};
use stm32f4xx_hal::{nb, prelude::*};
use crate::app;

const BUFFER_SIZE: usize = 256;

pub struct DataLogger {
    buffer0: [u8; BUFFER_SIZE],
    buffer1: [u8; BUFFER_SIZE],
    active_buffer: usize,
    active_buffer_index: usize,
    current_page_address: u32,
    logging_enabled: bool,
    bytes_logged: u32,
}

impl DataLogger {
    pub fn new() -> Self {
        Self {
            buffer0: [0u8; BUFFER_SIZE],
            buffer1: [0u8; BUFFER_SIZE],
            active_buffer: 0,
            active_buffer_index: 0,
            current_page_address: 0,
            logging_enabled: false,
            bytes_logged: 0,
        }
    }

    pub fn log_data_point(&mut self, data_point: DataPoint) {
        if !self.logging_enabled {
            return;
        }

        let buffer = if self.active_buffer == 0 {
            &mut self.buffer0
        } else {
            &mut self.buffer1
        };

        let mut data_buffer = [0u8; 16];
        let data_buffer = data_point.serialize(&mut data_buffer);

        self.bytes_logged += data_buffer.len() as u32;

        if self.active_buffer_index + data_buffer.len() < BUFFER_SIZE {
            for byte in data_buffer {
                buffer[self.active_buffer_index] = *byte;
                self.active_buffer_index += 1;
            }
        } else {
            let bytes_remaining = BUFFER_SIZE - self.active_buffer_index;
            for byte in &data_buffer[..bytes_remaining] {
                buffer[self.active_buffer_index] = *byte;
                self.active_buffer_index += 1;
            }

            self.active_buffer_index = 0;
            self.active_buffer = (self.active_buffer + 1) % 2;
            let buffer = if self.active_buffer == 0 {
                &mut self.buffer0
            } else {
                &mut self.buffer1
            };

            for byte in &data_buffer[bytes_remaining..] {
                buffer[self.active_buffer_index] = *byte;
                self.active_buffer_index += 1;
            }

            app::log_data_to_flash::spawn().unwrap();
        }
    }

    pub fn enable_logging(&mut self) {
        self.logging_enabled = true;
    }

    pub fn disable_logging(&mut self) {
        self.logging_enabled = false;
    }

    pub fn get_bytes_logged(&self) -> u32 {
        self.bytes_logged
    }
}

pub fn log_data_to_flash(mut ctx: app::log_data_to_flash::Context) {
    ctx.shared.data_logger.lock(|data_logger| {
        if data_logger.current_page_address >= 256 * 256 {
            defmt::error!("Data log flash is full");
            return;
        }

        let inactive_buffer = if data_logger.active_buffer == 0 {
            &mut data_logger.buffer1
        } else {
            &mut data_logger.buffer0
        };

        if let Err(err) = ctx.shared.w25x05.write_page(
            &mut ctx.shared.spi1,
            data_logger.current_page_address,
            inactive_buffer,
        ) {
            defmt::error!("Error writing log data to flash: {:?}", err);
        }

        data_logger.current_page_address += 256;
        for byte in inactive_buffer {
            *byte = 0;
        }
    });
}

pub fn read_log_page_and_transfer(ctx: app::read_log_page_and_transfer::Context, addr: u32) {
    let w25x05 = ctx.shared.w25x05;
    let spi1 = ctx.shared.spi1;
    let usart2_tx = ctx.local.usart2_tx;

    let mut buffer = [0u8; 256];
    if let Err(err) = w25x05.read_page(spi1, addr, &mut buffer) {
        defmt::error!("Error reading log page from flash: {:?}", err);
    }

    for byte in buffer {
        if let Err(_) = write!(usart2_tx, "{:02x}", byte) {
            defmt::error!("Error writing log page byte to USART2");
        }
    }

    if addr + 256 < 256 * 256 {
        app::read_log_page_and_transfer::spawn_after(10.millis().into(), addr + 256).unwrap();
    } else {
        defmt::info!("Finished transferring flash chip over USART.");
    }
}

pub fn usart2_interrupt(ctx: app::usart2_interrupt::Context) {
    let rx = ctx.local.usart2_rx;
    loop {
        match rx.read() {
            Ok(byte) => {
                if byte == 0x42 {
                    defmt::info!("USART2: Received command to transfer flash");

                    app::read_log_page_and_transfer::spawn(0).unwrap();
                } else {
                    defmt::warn!("USART2: Got unrecognized byte: {}", byte);
                }
            }
            Err(nb::Error::WouldBlock) => {
                break;
            }
            Err(nb::Error::Other(_)) => {
                defmt::error!("Error reading USART2");
                break;
            }
        }
    }
}

pub fn set_data_logging_state(mut ctx: app::set_data_logging_state::Context, state: bool) {
    ctx.shared.data_logger.lock(|data_logger| {
        if state {
            data_logger.enable_logging();
        } else {
            data_logger.disable_logging();
        }
    });
}

pub fn erase_data_log_flash(mut ctx: app::erase_data_log_flash::Context) {
    defmt::debug!("Erasing data log flash");

    ctx.shared.data_logger.lock(|data_logger| {
        data_logger.active_buffer = 0;
        data_logger.active_buffer_index = 0;
        data_logger.current_page_address = 0;
        data_logger.bytes_logged = 0;
        data_logger.logging_enabled = false;
    });

    ctx.shared.w25x05.erase_chip(&mut ctx.shared.spi1).unwrap();
    ctx.shared.w25x05.wait_for_not_busy(&mut ctx.shared.spi1);

    defmt::debug!("Data log flash erased");

    // Verify the flash has been erased
    let mut buffer = [0u8; 256];
    for page in 0..256 {
        if let Err(err) = ctx.shared.w25x05.read_page(&mut ctx.shared.spi1, page * 256, &mut buffer) {
            defmt::error!("Error reading log page from flash: {:?}", err);
        }

        for byte in &buffer {
            if *byte != 0xFF {
                defmt::error!("Flash erase failed at {}", page * 256);
                return;
            }
        }
    }

    defmt::debug!("Flash erase verified");
}