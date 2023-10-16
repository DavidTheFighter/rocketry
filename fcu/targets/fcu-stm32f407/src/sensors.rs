use crate::app;
use bmi088_rs::{AccelRange, GyroRange, Bmi088Accelerometer, Bmi088Gyroscope};
use hal::fcu_log;
use mint::Vector3;
use ms5611_rs::OversampleRatio;
use stm32f4xx_hal::prelude::*;
use rtic::Mutex;

pub fn bmi088_interrupt(mut ctx: app::bmi088_interrupt::Context) {
    let has_accel_data = ctx.local.accel_int_pin.check_interrupt();
    let has_gyro_data = ctx.local.gyro_int_pin.check_interrupt();

    let bmi088_accel = ctx.local.bmi088_accel;
    let bmi088_gyro = ctx.local.bmi088_gyro;
    let mut fcu = ctx.shared.fcu;

    fcu.lock(|fcu| {
        if has_accel_data {
            ctx.local.accel_int_pin.clear_interrupt_pending_bit();

            match bmi088_accel.read_data() {
                Ok((x, y, z)) => {
                    let data_point = fcu_log::DataPoint::Accelerometer { x, y, z };
                    ctx.shared.data_logger.lock(|data_logger| {
                        data_logger.log_data_point(data_point);
                    });

                    let raw_values = Vector3 { x, y, z };
                    let (x, y, z) = convert_raw_to_m_s2(bmi088_accel.get_range(), (x, y, z));
                    fcu.update_acceleration(Vector3 { x, y, z }, raw_values);
                },
                Err(_) => {
                    panic!("Error reading accelerometer data")
                }
            }
        }

        if has_gyro_data {
            ctx.local.gyro_int_pin.clear_interrupt_pending_bit();

            match bmi088_gyro.read_data() {
                Ok((x, y, z)) => {
                    let data_point = fcu_log::DataPoint::Gyroscope { x, y, z };
                    ctx.shared.data_logger.lock(|data_logger| {
                        data_logger.log_data_point(data_point);
                    });

                    let raw_values = Vector3 { x, y, z };
                    let (x, y, z) = convert_raw_to_rps(bmi088_gyro.get_range(), (x, y, z));
                    fcu.update_angular_velocity(Vector3 { x, y, z }, raw_values);
                },
                Err(_) => {
                    panic!("Error reading gyroscope data")
                }
            }
        }
    });
}

pub fn bmm150_interrupt(mut _ctx: app::bmm150_interrupt::Context) {
    // let has_mag_data = ctx.local.mag_int_pin.check_interrupt();

    // let bmm150 = ctx.local.bmm150;
    // let i2c1 = ctx.shared.i2c1;
    // let fcu = ctx.shared.fcu;

    // (i2c1, fcu).lock(|i2c1, fcu| {
    //     if has_mag_data {
    //         ctx.local.mag_int_pin.clear_interrupt_pending_bit();

    //         let (x, y, z, h) = bmm150.read_mag(i2c1);

    //         let data_point = fcu_log::DataPoint::Magnetometer {
    //             x: x as i16,
    //             y: y as i16,
    //             z: z as i16,
    //             h: h as i16,
    //         };
    //         ctx.shared.data_logger.lock(|data_logger| {
    //             data_logger.log_data_point(data_point);
    //         });

    //         let x = (x as f32) / 16.0;
    //         let y = (y as f32) / 16.0;
    //         let z = (z as f32) / 16.0;
    //         fcu.update_magnetic_field(Vector3 { x, y, z });
    //     }
    // });
}

pub fn ms5611_update(ctx: app::ms5611_update::Context) {
    let ms5611 = ctx.local.ms5611;
    let mut fcu = ctx.shared.fcu;

    fcu.lock(|fcu| {
        let delay_fn = |delay_ms| {
            let delay = (delay_ms as f32) * 0.001;
            cortex_m::asm::delay((delay * (app::MCU_FREQ as f32)) as u32);
        };
        match ms5611.read(OversampleRatio::Osr4096, delay_fn) {
            Ok((pressure, temperature)) => {
                // Units of pressure are in mbar * 100 which is equal to one pascal
                fcu.update_barometric_pressure(pressure as f32, (temperature as f32) * 0.01, pressure);
            },
            Err(_) => {
                panic!("Error reading pressure")
            }
        }
    });

    app::ms5611_update::spawn_after(100.millis().into()).unwrap();
}

fn convert_raw_to_m_s2(accel_range: AccelRange, raw_values: (i16, i16, i16)) -> (f32, f32, f32) {
    let (x_int16, y_int16, z_int16) = raw_values;

    let range = match accel_range {
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

fn convert_raw_to_rps(gyro_range: GyroRange, raw_values: (i16, i16, i16)) -> (f32, f32, f32) {
    let (x_int16, y_int16, z_int16) = raw_values;

    let range = match gyro_range {
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