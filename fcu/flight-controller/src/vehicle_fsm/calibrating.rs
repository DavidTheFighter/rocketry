use hal::{fcu_hal::VehicleState, comms_hal::{Packet, NetworkAddress}};
use nalgebra::Vector3;
use crate::{FiniteStateMachine, Fcu, state_vector::SensorCalibrationData};
use super::{Calibrating, FsmStorage};

impl FiniteStateMachine<VehicleState> for Calibrating {
    fn update(fcu: &mut Fcu, _dt: f32, _packets: &[(NetworkAddress, Packet)]) -> Option<VehicleState> {
        if Calibrating::calibration_time_ended(fcu) {
            Calibrating::update_calibration(fcu);

            return Some(VehicleState::Idle);
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
            barometric_altitude: 0.0,
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
        let accelerometer = fcu.sensor_data.accelerometer;
        let gyroscope = fcu.sensor_data.gyroscope;
        let magnetometer = fcu.sensor_data.magnetometer;
        let barometric_altitude = fcu.sensor_data.barometric_altitude;

        if let FsmStorage::Calibrating(storage) = &mut fcu.vehicle_fsm_storage {
            storage.accelerometer += Vector3::<f32>::from(accelerometer);
            storage.gyroscope += Vector3::<f32>::from(gyroscope);
            storage.magnetometer += Vector3::<f32>::from(magnetometer);
            storage.barometric_altitude += barometric_altitude;

            storage.data_count += 1;
        }
    }

    fn update_calibration(fcu: &mut Fcu) {
        if let FsmStorage::Calibrating(storage) = &mut fcu.vehicle_fsm_storage {
            let sensor_calibration = SensorCalibrationData {
                accelerometer: -storage.accelerometer / (storage.data_count as f32),
                gyroscope: -storage.gyroscope / (storage.data_count as f32),
                magnetometer: -storage.magnetometer / (storage.data_count as f32),
                barometric_altitude: -storage.barometric_altitude / (storage.data_count as f32),
            };

            fcu.state_vector.update_calibration(sensor_calibration);
        }
    }
}