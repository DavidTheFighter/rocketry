use crate::app;
use bmi088_rs::{AccelRange, GyroRange, Bmi088Accelerometer, Bmi088Gyroscope};
use shared::fcu_hal::FcuSensorData;
use mint::Vector3;
use ms5611_rs::OversampleRatio;
use stm32f4xx_hal::prelude::*;
use rtic::Mutex;
use ublox::{Parser, FixedLinearBuffer};

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
                    let raw_data = Vector3 { x, y, z };
                    let (x, y, z) = convert_raw_to_m_s2(bmi088_accel.get_range(), (x, y, z));
                    fcu.update_sensor_data(FcuSensorData::Accelerometer {
                        acceleration: Vector3 { x, y, z },
                        raw_data,
                    });
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
                    let raw_data = Vector3 { x, y, z };
                    let (x, y, z) = convert_raw_to_rps(bmi088_gyro.get_range(), (x, y, z));
                    fcu.update_sensor_data(FcuSensorData::Gyroscope {
                        angular_velocity: Vector3 { x, y, z },
                        raw_data,
                    });
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

pub fn ublox_update(ctx: app::ublox_update::Context) {
    let mut buffer = [0u8; 256];
    let mut buffer2 = [0u8; 256];
    let ublox_buffer = FixedLinearBuffer::new(&mut buffer);
    let mut parser = Parser::new(ublox_buffer);

    if ctx.local.uart4.is_rx_not_empty() {
        let mut num_bytes = 0;
        loop {
            if !ctx.local.uart4.is_rx_not_empty() {
                break;
            }

            let byte = ctx.local.uart4.read().unwrap();
            buffer2[num_bytes] = byte;
            num_bytes += 1;
        }

        let mut it = parser.consume(&buffer2[..num_bytes]);
        loop {
            match it.next() {
                Some(Ok(packet)) => {
                    match packet {
                        ublox::PacketRef::NavPosLlh(_) => todo!(),
                        ublox::PacketRef::NavStatus(_) => todo!(),
                        ublox::PacketRef::NavDop(_) => todo!(),
                        ublox::PacketRef::NavPvt(_) => todo!(),
                        ublox::PacketRef::NavSolution(_) => todo!(),
                        ublox::PacketRef::NavVelNed(_) => todo!(),
                        ublox::PacketRef::NavHpPosLlh(_) => todo!(),
                        ublox::PacketRef::NavHpPosEcef(_) => todo!(),
                        ublox::PacketRef::NavTimeUTC(_) => todo!(),
                        ublox::PacketRef::NavTimeLs(_) => todo!(),
                        ublox::PacketRef::NavSat(_) => todo!(),
                        ublox::PacketRef::NavEoe(_) => todo!(),
                        ublox::PacketRef::NavOdo(_) => todo!(),
                        ublox::PacketRef::CfgOdo(_) => todo!(),
                        ublox::PacketRef::MgaAck(_) => todo!(),
                        ublox::PacketRef::MgaGpsIono(_) => todo!(),
                        ublox::PacketRef::MgaGpsEph(_) => todo!(),
                        ublox::PacketRef::MgaGloEph(_) => todo!(),
                        ublox::PacketRef::AlpSrv(_) => todo!(),
                        ublox::PacketRef::AckAck(_) => todo!(),
                        ublox::PacketRef::AckNak(_) => todo!(),
                        ublox::PacketRef::CfgItfm(_) => todo!(),
                        ublox::PacketRef::CfgPrtI2c(_) => todo!(),
                        ublox::PacketRef::CfgPrtSpi(_) => todo!(),
                        ublox::PacketRef::CfgPrtUart(_) => todo!(),
                        ublox::PacketRef::CfgNav5(_) => todo!(),
                        ublox::PacketRef::CfgAnt(_) => todo!(),
                        ublox::PacketRef::CfgTmode2(_) => todo!(),
                        ublox::PacketRef::CfgTmode3(_) => todo!(),
                        ublox::PacketRef::CfgTp5(_) => todo!(),
                        ublox::PacketRef::InfError(_) => todo!(),
                        ublox::PacketRef::InfWarning(_) => todo!(),
                        ublox::PacketRef::InfNotice(_) => todo!(),
                        ublox::PacketRef::InfTest(_) => todo!(),
                        ublox::PacketRef::InfDebug(_) => todo!(),
                        ublox::PacketRef::RxmRawx(_) => todo!(),
                        ublox::PacketRef::TimTp(_) => todo!(),
                        ublox::PacketRef::TimTm2(_) => todo!(),
                        ublox::PacketRef::MonVer(_) => todo!(),
                        ublox::PacketRef::MonGnss(_) => todo!(),
                        ublox::PacketRef::MonHw(_) => todo!(),
                        ublox::PacketRef::RxmRtcm(_) => todo!(),
                        ublox::PacketRef::EsfMeas(_) => todo!(),
                        ublox::PacketRef::EsfIns(_) => todo!(),
                        ublox::PacketRef::HnrAtt(_) => todo!(),
                        ublox::PacketRef::HnrIns(_) => todo!(),
                        ublox::PacketRef::HnrPvt(_) => todo!(),
                        ublox::PacketRef::NavAtt(_) => todo!(),
                        ublox::PacketRef::NavClock(_) => todo!(),
                        ublox::PacketRef::NavVelECEF(_) => todo!(),
                        ublox::PacketRef::MgaGpsEPH(_) => todo!(),
                        ublox::PacketRef::RxmSfrbx(_) => todo!(),
                        ublox::PacketRef::EsfRaw(_) => todo!(),
                        ublox::PacketRef::TimSvin(_) => todo!(),
                        ublox::PacketRef::SecUniqId(_) => todo!(),
                        ublox::PacketRef::Unknown(_) => todo!(),
                    }
                },
                Some(Err(_)) => {
                    defmt::error!("Error parsing ublox packet");
                },
                None => {
                    defmt::error!("No ublox packet found");
                    break;
                }
            }
        }
    }

    app::ms5611_update::spawn_after(10.millis().into()).unwrap();
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
                fcu.update_sensor_data(FcuSensorData::Barometer {
                    pressure: pressure as f32,
                    temperature: (temperature as f32) * 0.01,
                    raw_data: pressure,
                });
            },
            Err(_) => {
                panic!("Error reading pressure")
            }
        }
    });

    app::ms5611_update::spawn_after(100.millis().into()).unwrap();
}

pub fn adc1_dma2_stream0_interrupt(ctx: app::adc1_dma2_stream0_interrupt::Context) {
    
}

fn convert_raw_to_m_s2(accel_range: AccelRange, raw_values: (i16, i16, i16)) -> (f32, f32, f32) {
    let (x_int16, y_int16, z_int16) = raw_values;

    let range = match accel_range {
        AccelRange::G3 => 3.0,
        AccelRange::G6 => 6.0,
        AccelRange::G12 => 12.0,
        AccelRange::G24 => 24.0,
    };

    let scale = 9.80665 * range / 32768.0;

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