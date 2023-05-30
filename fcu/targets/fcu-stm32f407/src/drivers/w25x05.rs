use cortex_m::prelude::{_embedded_hal_blocking_spi_Write, _embedded_hal_blocking_spi_Transfer};
use stm32f4xx_hal::{hal::digital::v2::OutputPin, spi};

use crate::app;

pub struct W25X05<CSN, HOLD> {
    csn: CSN,
    hold: HOLD,
}

impl<CSN: OutputPin, HOLD: OutputPin> W25X05<CSN, HOLD> {
    pub fn new(csn: CSN, hold: HOLD) -> Self {
        Self {
            csn,
            hold,
        }
    }

    pub fn write_page<SPIx: spi::Instance, PINS>(
        &mut self,
        spi: &mut spi::Spi<SPIx, PINS, false>,
        address: u32,
        data: &[u8],
    ) -> Result<(), ()> {
        self.wait_for_not_busy(spi);

        let address = address.to_ne_bytes();
        let buffer = [PAGE_PROGRAM, address[2], address[1], address[0]];

        self.enable_write(spi);

        cortex_m::asm::delay(100_000);

        self.csn.set_low();
        spi.write(&buffer).unwrap();
        spi.write(data).unwrap();
        self.csn.set_high();

        Ok(())
    }

    pub fn read_page<SPIx: spi::Instance, PINS>(
        &mut self,
        spi: &mut spi::Spi<SPIx, PINS, false>,
        address: u32,
        data: &mut [u8],
    ) -> Result<(), ()> {
        self.wait_for_not_busy(spi);

        let address = address.to_ne_bytes();
        let buffer = [READ_DATA, address[2], address[1], address[0]];

        self.csn.set_low();
        spi.write(&buffer).unwrap();
        spi.transfer(data).unwrap();
        self.csn.set_high();

        Ok(())
    }

    pub fn erase_sector<SPIx: spi::Instance, PINS>(
        &mut self,
        spi: &mut spi::Spi<SPIx, PINS, false>,
        address: u32,
    ) -> Result<(), ()> {
        self.wait_for_not_busy(spi);

        let address = address.to_ne_bytes();
        let buffer = [SECTOR_ERASE, address[2], address[1], address[0]];

        self.enable_write(spi);

        self.csn.set_low();
        spi.write(&buffer).unwrap();
        self.csn.set_high();

        Ok(())
    }

    pub fn erase_chip<SPIx: spi::Instance, PINS>(
        &mut self,
        spi: &mut spi::Spi<SPIx, PINS, false>,
    ) -> Result<(), ()> {
        self.wait_for_not_busy(spi);
        self.enable_write(spi);

        self.csn.set_low();
        spi.write(&[CHIP_ERASE]).unwrap();
        self.csn.set_high();

        Ok(())
    }

    pub fn get_unique_id<SPIx: spi::Instance, PINS>(&mut self, spi: &mut spi::Spi<SPIx, PINS, false>) -> u64 {
        self.wait_for_not_busy(spi);

        let mut buffer = [0x00_u8; 13];
        buffer[0] = READ_UNIQUE_ID;

        self.csn.set_low();
        spi.transfer(&mut buffer).unwrap();
        self.csn.set_high();

        buffer[5..].iter().fold(0, |acc, &x| (acc << 8) | x as u64)
    }

    pub fn wait_for_not_busy<SPIx: spi::Instance, PINS>(
        &mut self,
        spi: &mut spi::Spi<SPIx, PINS, false>
    ) {
        loop {
            let mut buffer = [0_u8; 2];
            buffer[0] = READ_STATUS_REGISTER;

            self.csn.set_low();
            spi.transfer(&mut buffer).unwrap();
            self.csn.set_high();

            if buffer[1] & 0x01 == 0 {
                break;
            }
        }
    }

    fn enable_write<SPIx: spi::Instance, PINS>(
        &mut self,
        spi: &mut spi::Spi<SPIx, PINS, false>
    ) {
        self.csn.set_low();
        spi.write(&[WRITE_ENABLE]).unwrap();
        self.csn.set_high();
    }
}

const PAGE_PROGRAM: u8 = 0x02;
const READ_DATA: u8 = 0x03;
const READ_STATUS_REGISTER: u8 = 0x05;
const WRITE_ENABLE: u8 = 0x06;
const SECTOR_ERASE: u8 = 0x20;
const READ_UNIQUE_ID: u8 = 0x4B;
const CHIP_ERASE: u8 = 0x60;