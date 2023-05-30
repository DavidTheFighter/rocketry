pub mod bmi088;
pub mod bmm150;
pub mod w25x05;

use stm32f4xx_hal::i2c;

fn i2c_write_validate<I2Cx: i2c::Instance, PINS>(
    i2c: &mut i2c::I2c<I2Cx, PINS>,
    addr: u8,
    reg: u8,
    value: u8,
) -> Result<(), (u8, u8)> {
    let mut buf = [0; 1];

    if i2c.write(addr, &[reg, value]).is_err() {
        return Err((0, 0));
    }

    if i2c.write_read(addr, &[reg], &mut buf).is_err() {
        return Err((0, 0));
    }

    if buf[0] != value {
        Err((buf[0], value))
    } else {
        Ok(())
    }
}