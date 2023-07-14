use crate::app;
use hal::fcu_log;
use mint::Vector3;
use stm32f4xx_hal::prelude::*;
use rtic::{mutex_prelude::TupleExt02, Mutex};

pub fn bmi088_interrupt(mut ctx: app::bmi088_interrupt::Context) {
    let has_accel_data = ctx.local.accel_int_pin.check_interrupt();
    let has_gyro_data = ctx.local.gyro_int_pin.check_interrupt();

    let bmi088 = ctx.local.bmi088;
    let i2c1 = ctx.shared.i2c1;
    let fcu = ctx.shared.fcu;

    (i2c1, fcu).lock(|i2c1, fcu| {
        if has_accel_data {
            ctx.local.accel_int_pin.clear_interrupt_pending_bit();

            let (x, y, z) = bmi088.get_raw_accel(i2c1);
            let data_point = fcu_log::DataPoint::Accelerometer { x, y, z };
            ctx.shared.data_logger.lock(|data_logger| {
                data_logger.log_data_point(data_point);
            });

            let (x, y, z) = bmi088.convert_raw_to_m_s2((x, y, z));
            fcu.update_acceleration(Vector3 { x, y, z });
        }

        if has_gyro_data {
            ctx.local.gyro_int_pin.clear_interrupt_pending_bit();

            let (x, y, z) = bmi088.read_raw_gyro(i2c1);
            let data_point = fcu_log::DataPoint::Gyroscope { x, y, z };
            ctx.shared.data_logger.lock(|data_logger| {
                data_logger.log_data_point(data_point);
            });

            let (x, y, z) = bmi088.convert_raw_to_rps((x, y, z));
            fcu.update_angular_velocity(Vector3 { x, y, z });
        }
    });
}

pub fn bmm150_interrupt(mut ctx: app::bmm150_interrupt::Context) {
    let has_mag_data = ctx.local.mag_int_pin.check_interrupt();

    let bmm150 = ctx.local.bmm150;
    let i2c1 = ctx.shared.i2c1;
    let fcu = ctx.shared.fcu;

    (i2c1, fcu).lock(|i2c1, fcu| {
        if has_mag_data {
            ctx.local.mag_int_pin.clear_interrupt_pending_bit();

            let (x, y, z, h) = bmm150.read_mag(i2c1);

            let data_point = fcu_log::DataPoint::Magnetometer {
                x: x as i16,
                y: y as i16,
                z: z as i16,
                h: h as i16,
            };
            ctx.shared.data_logger.lock(|data_logger| {
                data_logger.log_data_point(data_point);
            });

            let x = (x as f32) / 16.0;
            let y = (y as f32) / 16.0;
            let z = (z as f32) / 16.0;
            fcu.update_magnetic_field(Vector3 { x, y, z });
        }
    });
}