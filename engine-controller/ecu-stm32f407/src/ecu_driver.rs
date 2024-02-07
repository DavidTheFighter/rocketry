use core::sync::atomic::Ordering;

use shared::SensorConfig;
use shared::comms_hal::{Packet, NetworkAddress};
use shared::ecu_hal::{EcuDriver, EcuSensor, EcuDAQFrame, EcuTelemetryFrame, IgniterState, FuelTankState, EcuSolenoidValve};
use rtic::Mutex;
use stm32f4xx_hal::gpio::{PA12, PA11, PA10, PA9, Output, PinState};
use stm32f4xx_hal::signature::{VtempCal30, VtempCal110};
use stm32f4xx_hal::timer::PwmChannel;
use stm32f4xx_hal::pac::TIM1;
use stm32f4xx_hal::prelude::*;
use strum::{IntoEnumIterator, EnumCount};

use crate::{app, now};

pub struct EcuControlPins {
    pub sv1_ctrl: PA12<Output>,
    pub sv2_ctrl: PA11<Output>,
    pub sv3_ctrl: PA10<Output>,
    pub sv4_ctrl: PA9<Output>,
    pub spark_ctrl: PwmChannel<TIM1, 0>,
}

pub struct Stm32F407EcuDriver {
    pins: EcuControlPins,
    sensor_configs: [SensorConfig; EcuSensor::COUNT],
    last_update_time: u64,
    cpu_utilization: u32,
    sensor_values: [f32; EcuSensor::COUNT],
    sensor_mins: [f32; EcuSensor::COUNT],
    sensor_maxs: [f32; EcuSensor::COUNT],
    solenoid_valve_states: [bool; EcuSolenoidValve::COUNT],
    sparking: bool,
}

impl Stm32F407EcuDriver {
    pub fn new(pins: EcuControlPins) -> Self {
        Self {
            pins,
            sensor_configs: [SensorConfig::default(); EcuSensor::COUNT],
            last_update_time: 0,
            cpu_utilization: 0,
            sensor_values: [0.0; EcuSensor::COUNT],
            sensor_mins: [0.0; EcuSensor::COUNT],
            sensor_maxs: [0.0; EcuSensor::COUNT],
            solenoid_valve_states: [false; EcuSolenoidValve::COUNT],
            sparking: false,
        }
    }

    pub fn update(&mut self, last_frame: EcuDAQFrame, mins: EcuDAQFrame, maxs: EcuDAQFrame) -> f32 {
        let current_time = now();
        let elapsed_time = ((current_time - self.last_update_time) as f32) * 1e-3;

        let apply_sensor_value = |sensor: EcuSensor, frame: EcuDAQFrame| -> f32 {
            self.sensor_configs[sensor as usize]
                .apply(frame.sensor_values[sensor as usize] as f32)
        };

        for sensor in EcuSensor::iter() {
            self.sensor_values[sensor as usize] = apply_sensor_value(sensor, last_frame);
            self.sensor_mins[sensor as usize] = apply_sensor_value(sensor, mins);
            self.sensor_maxs[sensor as usize] = apply_sensor_value(sensor, maxs);
        }

        self.sensor_values[EcuSensor::ECUBoardTemp as usize] =
            raw_board_temp_to_celsius(last_frame.sensor_values[EcuSensor::ECUBoardTemp as usize]);
        self.sensor_mins[EcuSensor::ECUBoardTemp as usize] =
            raw_board_temp_to_celsius(mins.sensor_values[EcuSensor::ECUBoardTemp as usize]);
        self.sensor_maxs[EcuSensor::ECUBoardTemp as usize] =
            raw_board_temp_to_celsius(maxs.sensor_values[EcuSensor::ECUBoardTemp as usize]);

        self.last_update_time = current_time;

        elapsed_time
    }
}

impl EcuDriver for Stm32F407EcuDriver {
    fn set_solenoid_valve(&mut self, valve: EcuSolenoidValve, state: bool) {
        let pin_state = if state { PinState::High } else { PinState::Low };

        match valve {
            EcuSolenoidValve::IgniterFuelMain => self.pins.sv1_ctrl.set_state(pin_state),
            EcuSolenoidValve::IgniterOxidizerMain => self.pins.sv2_ctrl.set_state(pin_state),
            EcuSolenoidValve::FuelPress => self.pins.sv3_ctrl.set_state(pin_state),
            EcuSolenoidValve::FuelVent => self.pins.sv4_ctrl.set_state(pin_state),
        }

        self.solenoid_valve_states[valve as usize] = state;
    }

    fn set_sparking(&mut self, state: bool) {
        if state {
            self.pins.spark_ctrl.enable();
            self.pins
                .spark_ctrl
                .set_duty(self.pins.spark_ctrl.get_max_duty() / 8);
        } else {
            self.pins.spark_ctrl.disable();
            self.pins.spark_ctrl.set_duty(0);
        }

        self.sparking = state;
    }

    fn get_solenoid_valve(&self, valve: EcuSolenoidValve) -> bool {
        self.solenoid_valve_states[valve as usize]
    }

    fn get_sensor(&self, sensor: EcuSensor) -> f32 {
        self.sensor_values[sensor as usize]
    }

    fn get_sparking(&self) -> bool {
        self.sparking
    }

    fn send_packet(&mut self, packet: Packet, address: NetworkAddress) {
        app::send_packet::spawn(packet, address).expect("ecu_driver failed to send a packet");
    }

    fn generate_telemetry_frame(&self) -> EcuTelemetryFrame {
        EcuTelemetryFrame {
            timestamp: now(),
            igniter_state: IgniterState::Idle,
            fuel_tank_state: FuelTankState::Idle,
            sensors: self.sensor_values,
            solenoid_valves: self.solenoid_valve_states,
            sparking: self.sparking,
            cpu_utilization: self.cpu_utilization,
        }
    }

    fn collect_daq_sensor_data(&mut self, sensor: EcuSensor) -> (f32, f32, f32) {
        (
            self.sensor_values[sensor as usize],
            self.sensor_mins[sensor as usize],
            self.sensor_maxs[sensor as usize],
        )
    }

    fn configure_sensor(&mut self, sensor: EcuSensor, config: SensorConfig) {
        self.sensor_configs[sensor as usize] = config;
    }

    fn as_mut_any(&mut self) -> &mut dyn core::any::Any {
        self
    }
}

pub fn ecu_update(mut ctx: app::ecu_update::Context) {
    app::ecu_update::spawn_after(10.millis().into()).unwrap();

    let (frame, mins, maxs) = ctx.shared.daq.lock(|daq| {
        let (frame, mins, maxs) = daq.get_values();
        daq.reset_ranges();

        (frame, mins, maxs)
    });

    let cpu_utilization = ctx.shared.cpu_utilization.load(Ordering::Relaxed);
    ctx.local.ecu_module.driver().cpu_utilization = cpu_utilization;

    let dt = ctx.local.ecu_module.driver().update(frame, mins, maxs);
    ctx.local.ecu_module.update(dt, None);

    ctx.shared.packet_queue.lock(|packet_queue| {
        while let Some(packet) = packet_queue.dequeue() {
            ctx.local.ecu_module.update(0.0, Some(packet));
        }
    });
}

fn raw_board_temp_to_celsius(sample: u16) -> f32 {
    let cal30 = VtempCal30::get().read() as f32;
    let cal110 = VtempCal110::get().read() as f32;

    (110.0 - 30.0) * ((sample as f32) - cal30) / (cal110 - cal30) + 30.0
}
