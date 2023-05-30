use stm32f4xx_hal::i2c;

use super::i2c_write_validate;

#[derive(Debug, Copy, Clone)]
pub enum MagDataRate {
    Hz10 = 0x00,
    Hz2 = 0x01,
    Hz6 = 0x02,
    Hz8 = 0x03,
    Hz15 = 0x04,
    Hz20 = 0x05,
    Hz25 = 0x06,
    Hz30 = 0x07,
}

pub struct Bmm150 {
    addr: u8,
    data_rate: MagDataRate,
}

impl Bmm150 {
    pub fn new(addr: u8, data_rate: MagDataRate) -> Self {
        Self {
            addr,
            data_rate,
        }
    }

    pub fn read_mag<I2Cx: i2c::Instance, PINS>(&self, i2c: &mut i2c::I2c<I2Cx, PINS>) -> (i16, i16, i16, i16) {
        let mut buffer = [0u8; 8];

        if let Err(err) = i2c.write_read(self.addr, &[MAG_DATA], &mut buffer) {
            // hprintln!("Error reading accel: {:?}", err);
        }

        let x_int16 = ((buffer[1] as i16) << 8) | (buffer[0] as i16);
        let y_int16 = ((buffer[3] as i16) << 8) | (buffer[2] as i16);
        let z_int16 = ((buffer[5] as i16) << 8) | (buffer[4] as i16);
        let h_int16 = ((buffer[7] as i16) << 8) | (buffer[6] as i16);

        (x_int16, y_int16, z_int16, h_int16)
    }

    pub fn turn_on<I2Cx: i2c::Instance, PINS>(&self, i2c: &mut i2c::I2c<I2Cx, PINS>) {
        let reg =
            (0x0 << 1) | // Normal mode
            ((self.data_rate as u8) << 3) |
        0;

        // Put the magnetometer into normal mode and set the data rate
        if let Err(err) = i2c_write_validate(i2c, self.addr, MAG_CTRL, reg) {
            // hprintln!("Error turning on mag: {:?}", err);
        }

        // Enable the data ready pin
        if let Err(err) = i2c_write_validate(i2c, self.addr, MAG_INT_CTRL, 0b1000_0100) {
            // hprintln!("Error configuring mag: {:?}", err);
        }

        // Configure the XY repetitions
        if let Err(err) = i2c_write_validate(i2c, self.addr, MAG_XY_REP, 23) {
            // hprintln!("Error configuring mag xy rep: {:?}", err);
        }

        // Configure the Z repetitions
        if let Err(err) = i2c_write_validate(i2c, self.addr, MAG_Z_REP, 41) {
            // hprintln!("Error configuring mag z rep: {:?}", err);
        }
    }

    pub fn reset<I2Cx: i2c::Instance, PINS>(&self, i2c: &mut i2c::I2c<I2Cx, PINS>) {
        // If already powered on, this will soft reset into sleep mode. On first startup
        // this won't do anything
        if let Err(err) = i2c.write(self.addr, &[MAG_CTRL, 0b1000_0010]) {
            // hprintln!("Error resetting mag: {:?}", err);
        }

        // Delay for a bit to let the chip reset
        cortex_m::asm::delay(200000);

        // This will put the chip into sleep mode if it isn't already
        if let Err(err) = i2c.write(self.addr, &[MAG_RST_CTRL, 0b0000_0001]) {
            // hprintln!("Error resetting mag: {:?}", err);
        }
    }
}

const MAG_DATA: u8 = 0x42;
const MAG_RST_CTRL: u8 = 0x4B;
const MAG_CTRL: u8 = 0x4C;
const MAG_INT_CTRL: u8 = 0x4E;
const MAG_XY_REP: u8 = 0x51;
const MAG_Z_REP: u8 = 0x52;