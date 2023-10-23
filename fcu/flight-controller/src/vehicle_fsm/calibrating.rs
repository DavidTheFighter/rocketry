use hal::{fcu_hal::VehicleState, comms_hal::{Packet, NetworkAddress}};
use nalgebra::Vector3;
use crate::{FiniteStateMachine, Fcu, state_vector::SensorCalibrationData};
use super::{Calibrating, FsmStorage};

impl FiniteStateMachine<VehicleState> for Calibrating {
    fn update(fcu: &mut Fcu, _dt: f32, _packets: &[(NetworkAddress, Packet)]) -> Option<VehicleState> {
        if Calibrating::calibration_time_ended(fcu) {
            Calibrating::update_calibration(fcu);

            return Some(VehicleState::Zeroing);
        }

        Calibrating::accumulate_sensor_data(fcu);

        None
    }

    fn setup_state(fcu: &mut Fcu) {
        fcu.vehicle_fsm_storage = FsmStorage::Calibrating(Calibrating {
            start_time: fcu.driver.timestamp(),
            accelerometer: Vector3::new(0.0, 0.0, 0.0),
            gyroscope: Vector3::new(0.0, 0.0, 0.0),
            magnetometer: Vector3::new(0.0, 0.0, 0.0),
            barometer_pressure: 0.0,
            data_count: 0,
        });
    }
}

impl Calibrating {
    fn calibration_time_ended(fcu: &mut Fcu) -> bool {
        let timestamp = fcu.driver.timestamp();

        if let FsmStorage::Calibrating(storage) = &mut fcu.vehicle_fsm_storage {
            let elapsed_time = timestamp - storage.start_time;

            return elapsed_time >= fcu.config.calibration_duration;
        }

        true
    }

    fn accumulate_sensor_data(fcu: &mut Fcu) {
        let accelerometer = fcu.state_vector.sensor_data.accelerometer;
        let gyroscope = fcu.state_vector.sensor_data.gyroscope;
        let magnetometer = fcu.state_vector.sensor_data.magnetometer;
        let barometer_pressure = fcu.state_vector.sensor_data.barometer_pressure;

        if let FsmStorage::Calibrating(storage) = &mut fcu.vehicle_fsm_storage {
            storage.accelerometer += Vector3::<f32>::from(accelerometer);
            storage.gyroscope += Vector3::<f32>::from(gyroscope);
            storage.magnetometer += Vector3::<f32>::from(magnetometer);
            storage.barometer_pressure += barometer_pressure;

            storage.data_count += 1;
        }
    }

    fn update_calibration(fcu: &mut Fcu) {
        if let FsmStorage::Calibrating(storage) = &mut fcu.vehicle_fsm_storage {
            let mut accelerometer_avg = storage.accelerometer / (storage.data_count as f32);
            // let acceleration_by_gravity = accelerometer_avg.normalize() * 9.80665;

            // accelerometer_avg -= acceleration_by_gravity;

            let sensor_calibration = SensorCalibrationData {
                accelerometer: -accelerometer_avg,
                gyroscope: -storage.gyroscope / (storage.data_count as f32),
                magnetometer: -storage.magnetometer / (storage.data_count as f32),
                barometer_pressure: -storage.barometer_pressure / (storage.data_count as f32),
            };

            fcu.state_vector.update_calibration(sensor_calibration);
        }
    }
}