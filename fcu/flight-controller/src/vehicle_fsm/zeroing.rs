use hal::{fcu_hal::VehicleState, comms_hal::{Packet, NetworkAddress}};
use nalgebra::{Vector3, UnitVector3, UnitQuaternion};
use crate::{FiniteStateMachine, Fcu, state_vector::SensorCalibrationData};
use super::{Zeroing, FsmStorage};
use num_traits::Float;

impl FiniteStateMachine<VehicleState> for Zeroing {
    fn update(fcu: &mut Fcu, _dt: f32, _packets: &[(NetworkAddress, Packet)]) -> Option<VehicleState> {
        if Zeroing::zeroing_time_ended(fcu) {
            Zeroing::finish_zeroing(fcu);

            return Some(VehicleState::Idle);
        }

        Zeroing::accumulate_sensor_data(fcu);

        None
    }

    fn setup_state(fcu: &mut Fcu) {
        fcu.vehicle_fsm_storage = FsmStorage::Zeroing(Zeroing {
            start_time: fcu.driver.timestamp(),
            accelerometer: Vector3::new(0.0, 0.0, 0.0),
        });
    }
}

impl Zeroing {
    fn zeroing_time_ended(fcu: &mut Fcu) -> bool {
        let timestamp = fcu.driver.timestamp();

        if let FsmStorage::Zeroing(storage) = &mut fcu.vehicle_fsm_storage {
            let elapsed_time = timestamp - storage.start_time;

            return elapsed_time >= fcu.config.calibration_duration;
        }

        true
    }

    fn accumulate_sensor_data(fcu: &mut Fcu) {
        let accelerometer = fcu.state_vector.sensor_data.accelerometer;

        if let FsmStorage::Zeroing(storage) = &mut fcu.vehicle_fsm_storage {
            storage.accelerometer += Vector3::<f32>::from(accelerometer) + fcu.state_vector.sensor_calibration.accelerometer;
        }
    }

    fn finish_zeroing(fcu: &mut Fcu) {
        if let FsmStorage::Zeroing(storage) = &mut fcu.vehicle_fsm_storage {
            let down = UnitVector3::new_normalize(Vector3::new(0.0, -1.0, 0.0).normalize());
            let measured_down = UnitVector3::new_normalize(storage.accelerometer.normalize());

            let dot = down.dot(&measured_down);
            let angle = dot.acos();

            if angle.abs() < f32::EPSILON {
                let zeroed_orientation = UnitQuaternion::identity();

                fcu.state_vector.position_filter.zero();
                fcu.state_vector.orientation_filter.zero(zeroed_orientation);
            } else {
                let axis = UnitVector3::new_normalize(down.cross(&measured_down));

                let zeroed_orientation = UnitQuaternion::from_axis_angle(&axis, angle);

                fcu.state_vector.position_filter.zero();
                fcu.state_vector.orientation_filter.zero(zeroed_orientation);
            }
        }
    }
}
