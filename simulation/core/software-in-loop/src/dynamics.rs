use nalgebra::{Vector3, UnitQuaternion, Quaternion};
use pyo3::{prelude::*, types::PyList, exceptions::PyIndexError};

type Scalar = f64;

const G: Scalar = -9.806;

#[pyclass]
pub struct Dynamics {
    position: Vector3<Scalar>,     // World frame
    velocity: Vector3<Scalar>,     // World frame
    acceleration_world_frame: Vector3<Scalar>,
    acceleration_body_frame: Vector3<Scalar>,
    orientation: UnitQuaternion<Scalar>,
    angular_velocity: Vector3<Scalar>,
    angular_acceleration: Vector3<Scalar>,
    motor_thrust: Vector3<Scalar>,  // Body frame
    #[pyo3(get, set)]
    landed: bool,
}

#[pymethods]
impl Dynamics {
    pub fn update(&mut self, dt: f64) {
        let dt = dt as Scalar;

        let gravity = Vector3::new(0.0, G, 0.0);
        let gravity_accel_body_frame = self.orientation.inverse() * gravity;

        self.acceleration_body_frame = self.motor_thrust.clone();
        if !self.landed {
            self.acceleration_body_frame += gravity_accel_body_frame;
        }

        self.acceleration_world_frame = self.orientation * self.acceleration_body_frame;

        // Time step
        self.velocity += self.acceleration_world_frame * dt;
        self.position += self.velocity * dt;

        self.angular_velocity += self.angular_acceleration * dt;
        self.orientation = integrate_angular_velocity_rk4(self.orientation, self.angular_velocity, dt);
    }

    #[new]
    pub fn new() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            acceleration_world_frame: Vector3::new(0.0, 0.0, 0.0),
            acceleration_body_frame: Vector3::new(0.0, 0.0, 0.0),
            orientation: UnitQuaternion::identity(),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
            angular_acceleration: Vector3::new(0.0, 0.0, 0.0),
            motor_thrust: Vector3::new(0.0, 0.0, 0.0),
            landed: true,
        }
    }

    #[getter(position)]
    pub fn get_position(&self) -> PyResult<Vec<f64>> {
        Ok(self.position.iter().map(|x| *x).collect())
    }

    #[setter(position)]
    pub fn set_position(&mut self, list: &PyList) -> PyResult<()> {
        set_vec3(&mut self.position, list)
    }

    #[getter(velocity)]
    pub fn get_velocity(&self) -> PyResult<Vec<f64>> {
        Ok(self.velocity.iter().map(|x| *x).collect())
    }

    #[setter(velocity)]
    pub fn set_velocity(&mut self, list: &PyList) -> PyResult<()> {
        set_vec3(&mut self.velocity, list)
    }

    #[getter(acceleration_world_frame)]
    pub fn get_acceleration_world_frame(&self) -> PyResult<Vec<f64>> {
        Ok(self.acceleration_world_frame.iter().map(|x| *x).collect())
    }

    #[setter(acceleration_world_frame)]
    pub fn set_acceleration_world_frame(&mut self, list: &PyList) -> PyResult<()> {
        set_vec3(&mut self.acceleration_world_frame, list)
    }

    #[getter(acceleration_body_frame)]
    pub fn get_acceleration_body_frame(&self) -> PyResult<Vec<f64>> {
        Ok(self.acceleration_body_frame.iter().map(|x| *x).collect())
    }

    #[setter(acceleration_body_frame)]
    pub fn set_acceleration_body_frame(&mut self, list: &PyList) -> PyResult<()> {
        set_vec3(&mut self.acceleration_body_frame, list)
    }

    #[getter(orientation)]
    pub fn get_orientation(&self) -> PyResult<Vec<f64>> {
        Ok(vec![
            self.orientation.quaternion().w,
            self.orientation.quaternion().i,
            self.orientation.quaternion().j,
            self.orientation.quaternion().k,
        ])
    }

    #[setter(orientation)]
    pub fn set_orientation(&mut self, list: &PyList) -> PyResult<()> {
        if list.len() != 4 {
            return Err(PyIndexError::new_err("List length must be 4 for quaternion"));
        }

        self.orientation = UnitQuaternion::from_quaternion(Quaternion::new(
            list.get_item(0).unwrap().extract::<f64>().expect(".w wasn't a float"),
            list.get_item(1).unwrap().extract::<f64>().expect(".i wasn't a float"),
            list.get_item(2).unwrap().extract::<f64>().expect(".j wasn't a float"),
            list.get_item(3).unwrap().extract::<f64>().expect(".k wasn't a float"),
        ));

        Ok(())
    }

    #[getter(angular_velocity)]
    pub fn get_angular_velocity(&self) -> PyResult<Vec<f64>> {
        Ok(self.angular_velocity.iter().map(|x| *x).collect())
    }

    #[setter(angular_velocity)]
    pub fn set_angular_velocity(&mut self, list: &PyList) -> PyResult<()> {
        set_vec3(&mut self.angular_velocity, list)
    }

    #[getter(angular_acceleration)]
    pub fn get_angular_acceleration(&self) -> PyResult<Vec<f64>> {
        Ok(self.angular_acceleration.iter().map(|x| *x).collect())
    }

    #[setter(angular_acceleration)]
    pub fn set_angular_acceleration(&mut self, list: &PyList) -> PyResult<()> {
        set_vec3(&mut self.angular_acceleration, list)
    }

    #[getter(motor_thrust)]
    pub fn get_motor_thrust(&self) -> PyResult<Vec<f64>> {
        Ok(self.motor_thrust.iter().map(|x| *x).collect())
    }

    #[setter(motor_thrust)]
    pub fn set_motor_thrust(&mut self, list: &PyList) -> PyResult<()> {
        set_vec3(&mut self.motor_thrust, list)
    }
}

fn integrate_angular_velocity_rk4(
    quat: UnitQuaternion<Scalar>,
    ang_vel: Vector3<Scalar>,
    dt: Scalar,
) -> UnitQuaternion<Scalar> {
    let quat = quat.quaternion();
    let k1 = q_dot(&quat, ang_vel);
    let k2 = q_dot(&(quat + 0.5 * dt * k1), ang_vel);
    let k3 = q_dot(&(quat + 0.5 * dt * k2), ang_vel);
    let k4 = q_dot(&(quat + dt * k3), ang_vel);

    let q_deriv = (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
    let result = quat + q_deriv * dt;

    UnitQuaternion::from_quaternion(result)
}

fn q_dot(quat: &Quaternion<Scalar>, ang_vel: Vector3<Scalar>) -> Quaternion<Scalar> {
    Quaternion::new(
        0.0,
        0.5 * ang_vel.x * quat.i,
        0.5 * ang_vel.y * quat.j,
        0.5 * ang_vel.z * quat.k,
    )
}

fn set_vec3(vec: &mut Vector3<Scalar>, list: &PyList) -> PyResult<()> {
    if list.len() != 3 {
        return Err(PyIndexError::new_err("List length must be 3 for vec3"));
    }

   *vec = Vector3::new(
        list.get_item(0).unwrap().extract::<f64>().expect(".x wasn't a float"),
        list.get_item(1).unwrap().extract::<f64>().expect(".y wasn't a float"),
        list.get_item(2).unwrap().extract::<f64>().expect(".z wasn't a float"),
    );

    Ok(())
}