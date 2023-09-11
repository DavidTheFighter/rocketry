use stm32f4xx_hal::i2c;

use super::i2c_write_validate;

#[derive(Debug, Copy, Clone)]
pub enum AccelFilterBandwidth {
    OSR4 = 0x00,
    OSR2 = 0x01,
    Normal = 0x02,
}

#[derive(Debug, Copy, Clone)]
pub enum AccelDataRate {
    Hz12_5 = 0x05,
    Hz25 = 0x06,
    Hz50 = 0x07,
    Hz100 = 0x08,
    Hz200 = 0x09,
    Hz400 = 0x0A,
    Hz800 = 0x0B,
    Hz1600 = 0x0C,
}

#[derive(Debug, Copy, Clone)]
pub enum AccelRange {
    G3 = 0x00,
    G6 = 0x01,
    G12 = 0x02,
    G24 = 0x03,
}

#[derive(Debug, Copy, Clone)]
pub enum GyroRange {
    Deg2000 = 0x00,
    Deg1000 = 0x01,
    Deg500 = 0x02,
    Deg250 = 0x03,
    Deg125 = 0x04,
}

#[derive(Debug, Copy, Clone)]
pub enum GyroBandwidth {
    Data2000Filter532 = 0x00,
    Data2000Filter230 = 0x01,
    Data1000Filter116 = 0x02,
    Data400Filter47 = 0x03,
    Data200Filter23 = 0x04,
    Data100Filter12 = 0x05,
    Data200Filter64 = 0x06,
    Data100Filter32 = 0x07,
}

pub struct Bmi088 {
    accel_addr: u8,
    gyro_addr: u8,
    accel_range: AccelRange,
    gyro_range: GyroRange,
}

impl Bmi088 {
    pub fn new(accel_addr: u8, gyro_addr: u8) -> Self {
        Self {
            accel_addr,
            gyro_addr,
            accel_range: AccelRange::G6,
            gyro_range: GyroRange::Deg1000,
        }
    }

    pub fn configure<I2Cx: i2c::Instance, PINS>(
        &mut self,
        i2c: &mut i2c::I2c<I2Cx, PINS>,
        filter_bandwidth: AccelFilterBandwidth,
        data_rate: AccelDataRate,
        accel_range: AccelRange,
        gyro_range: GyroRange,
        gyro_bandwidth: GyroBandwidth,
    ) {
        self.accel_range = accel_range;
        self.gyro_range = gyro_range;

        let acc_conf_reg = 0x80 | ((filter_bandwidth as u8) << 4) | (data_rate as u8);
        let acc_range_reg = accel_range as u8;
        let acc_int1_io_conf_reg =
            (0x01 << 1) | // INT1 pin is active high
            (0x00 << 2) | // INT1 pin is push-pull
            (0x01 << 3) | // INT1 pin is an output
        0;
        let acc_int1_int2_map_data_reg = 0x01 << 2; // INT1 is mapped to data ready

        if let Err((reg, val)) = i2c_write_validate(i2c, self.accel_addr, ACC_CONF, acc_conf_reg) {
            // hprintln!("Error writing accel conf reg: {:?} should be {:?}", reg, val);
            loop{}
        }

        if let Err((reg, val)) = i2c_write_validate(i2c, self.accel_addr, ACC_RANGE, acc_range_reg) {
            // hprintln!("Error writing accel range reg: {:?} should be {:?}", reg, val);
            loop{}
        }

        if let Err((reg, val)) = i2c_write_validate(i2c, self.accel_addr, ACC_INT1_IO_CONF, acc_int1_io_conf_reg) {
            // hprintln!("Error writing accel int1_io_conf reg: {:?} should be {:?}", reg, val);
            loop{}
        }

        if let Err((reg, val)) = i2c_write_validate(i2c, self.accel_addr, ACC_INT1_INT2_MAP_DATA, acc_int1_int2_map_data_reg) {
            // hprintln!("Error writing accel int1_int2_map_data reg: {:?} should be {:?}", reg, val);
            loop{}
        }

        let gyro_range_reg = gyro_range as u8;
        let gyro_bandwidth_reg = gyro_bandwidth as u8;
        let gyro_int_ctrl_reg = 0x80; // Data ready interrupt enabled
        let gyro_int3_int4_io_conf_reg =
            (0x01 << 0) | // INT3 pin is active high
            (0x00 << 1) | // INT3 pin is push-pull
        0;
        let gyro_int3_int4_io_map_reg = 0x01; // Data ready mapped to int3

        if let Err((reg, val)) = i2c_write_validate(i2c, self.gyro_addr, GYRO_RANGE, gyro_range_reg) {
            // hprintln!("Error writing gyro rage reg: {:?} should be {:?}", reg, val);
            loop{}
        }

        if let Err((reg, val)) = i2c_write_validate(i2c, self.gyro_addr, GYRO_BANDWIDTH, gyro_bandwidth_reg) {
            // This register has bit #7 always set to 1, so we need to manually check

            if val != (reg & (!(1 << 7))) {
                // hprintln!("Error writing gyro bandwidth reg: {:?} should be {:?}", reg, val);
                loop{}
            }
        }

        if let Err((reg, val)) = i2c_write_validate(i2c, self.gyro_addr, GYRO_INT_CTRL, gyro_int_ctrl_reg) {
            // hprintln!("Error writing gyro int_ctrl reg: {:?} should be {:?}", reg, val);
            loop{}
        }

        if let Err((reg, val)) = i2c_write_validate(i2c, self.gyro_addr, GYRO_INT3_INT4_IO_CONF, gyro_int3_int4_io_conf_reg) {
            // hprintln!("Error writing gyro int3_int4_io_conf reg: {:?} should be {:?}", reg, val);
            loop{}
        }

        if let Err((reg, val)) = i2c_write_validate(i2c, self.gyro_addr, GYRO_INT3_INT4_IO_MAP, gyro_int3_int4_io_map_reg) {
            // hprintln!("Error writing gyro int3_int4_io_map reg: {:?} should be {:?}", reg, val);
            loop{}
        }

    }

    pub fn get_raw_accel<I2Cx: i2c::Instance, PINS>(&self, i2c: &mut i2c::I2c<I2Cx, PINS>) -> (i16, i16, i16) {
        let mut buffer = [0u8; 6];

        if let Err(err) = i2c.write_read(self.accel_addr, &[ACC_DATA], &mut buffer) {
            // hprintln!("Error reading accel: {:?}", err);
        }

        let x_int16 = ((buffer[1] as i16) << 8) + (buffer[0] as i16);
        let y_int16 = ((buffer[3] as i16) << 8) + (buffer[2] as i16);
        let z_int16 = ((buffer[5] as i16) << 8) + (buffer[4] as i16);

        (x_int16, y_int16, z_int16)
    }

    pub fn convert_raw_to_m_s2(&self, raw_values: (i16, i16, i16)) -> (f32, f32, f32) {
        let (x_int16, y_int16, z_int16) = raw_values;

        let range = match self.accel_range {
            AccelRange::G3 => 3.0,
            AccelRange::G6 => 6.0,
            AccelRange::G12 => 12.0,
            AccelRange::G24 => 24.0,
        };

        let scale = range / 32767.0;

        let x = x_int16 as f32 * scale;
        let y = y_int16 as f32 * scale;
        let z = z_int16 as f32 * scale;

        (x, y, z)
    }

    pub fn read_raw_gyro<I2Cx: i2c::Instance, PINS>(&self, i2c: &mut i2c::I2c<I2Cx, PINS>) -> (i16, i16, i16) {
        let mut buffer = [0u8; 6];

        if let Err(err) = i2c.write_read(self.gyro_addr, &[GYRO_DATA], &mut buffer) {
            // hprintln!("Error reading gyro: {:?}", err);
        }

        let x_int16 = ((buffer[1] as i16) << 8) + (buffer[0] as i16);
        let y_int16 = ((buffer[3] as i16) << 8) + (buffer[2] as i16);
        let z_int16 = ((buffer[5] as i16) << 8) + (buffer[4] as i16);

        (x_int16, y_int16, z_int16)
    }

    // Radians per second
    pub fn convert_raw_to_rps(&self, raw_values: (i16, i16, i16)) -> (f32, f32, f32) {
        let (x_int16, y_int16, z_int16) = raw_values;

        let range = match self.gyro_range {
            GyroRange::Deg125 => 125.0 * 3.1415926 / 180.0,
            GyroRange::Deg250 => 250.0 * 3.1415926 / 180.0,
            GyroRange::Deg500 => 500.0 * 3.1415926 / 180.0,
            GyroRange::Deg1000 => 1000.0 * 3.1415926 / 180.0,
            GyroRange::Deg2000 => 2000.0 * 3.1415926 / 180.0,
        };

        let x = (x_int16 as f32) / 32767.0 * range;
        let y = (y_int16 as f32) / 32767.0 * range;
        let z = (z_int16 as f32) / 32767.0 * range;

        (x, y, z)
    }

    pub fn turn_on<I2Cx: i2c::Instance, PINS>(&self, i2c: &mut i2c::I2c<I2Cx, PINS>) {
        if let Err(err) = i2c.write(self.accel_addr, &[ACC_PWR_CTRL, 0x04]) {
            // hprintln!("Error turning on accel: {:?}", err);
        }
    }

    pub fn reset<I2Cx: i2c::Instance, PINS>(&self, i2c: &mut i2c::I2c<I2Cx, PINS>) {
        if let Err(err) = i2c.write(self.accel_addr, &[ACC_SOFTRESET, 0xB6]) {
            // hprintln!("Error resetting accel: {:?}", err);
        }

        if let Err(err) = i2c.write(self.gyro_addr, &[GYRO_SOFTRESET, 0xB6]) {
            // hprintln!("Error resetting accel: {:?}", err);
        }
    }

    pub fn get_accel_chip_id<I2Cx: i2c::Instance, PINS>(&self, i2c: &mut i2c::I2c<I2Cx, PINS>) -> u8 {
        let mut buffer = [0u8; 1];
        if let Err(err) = i2c.write_read(self.accel_addr, &[0x00], &mut buffer) {
            // hprintln!("Error reading accel chip id: {:?}", err);
        }
        buffer[0]
    }

    pub fn get_gyro_chip_id<I2Cx: i2c::Instance, PINS>(&self, i2c: &mut i2c::I2c<I2Cx, PINS>) -> u8 {
        let mut buffer = [0u8; 1];
        if let Err(err) = i2c.write_read(self.gyro_addr, &[0x00], &mut buffer) {
            // hprintln!("Error reading gyro chip id: {:?}", err);
        }
        buffer[0]
    }
}

const ACC_DATA: u8 = 0x12;
const ACC_CONF: u8 = 0x40;
const ACC_RANGE: u8 = 0x41;
const ACC_INT1_IO_CONF: u8 = 0x53;
const ACC_INT1_INT2_MAP_DATA: u8 = 0x58;
const ACC_PWR_CTRL: u8 = 0x7D;
const ACC_SOFTRESET: u8 = 0x7E;

const GYRO_DATA: u8 = 0x02;
const GYRO_RANGE: u8 = 0x0F;
const GYRO_BANDWIDTH: u8 = 0x10;
const GYRO_SOFTRESET: u8 = 0x14;
const GYRO_INT_CTRL: u8 = 0x15;
const GYRO_INT3_INT4_IO_CONF: u8 = 0x16;
const GYRO_INT3_INT4_IO_MAP: u8 = 0x18;