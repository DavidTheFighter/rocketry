mod igniter_fsm;
mod fuel_tank_fsm;

use rtic::Mutex;
use stm32_eth::stm32::TIM1;
use stm32f4xx_hal::{
    prelude::*, 
    signature::{VtempCal30, VtempCal110}, 
    gpio::{PA9, PA10, PA11, PA12, Output, PinState}, timer::PwmChannel
};

use hal::{ecu_hal::{ECUConfiguration, ECUSensor, IgniterState, FuelTankState, ECUTelemetryFrame}, comms_hal::{Packet, NetworkAddress}};

use crate::app;

use self::{igniter_fsm::IgniterStateStorage, fuel_tank_fsm::FuelTankStateStorage};

pub struct ECUState {
    config: ECUConfiguration,
    igniter_state: IgniterState,
    igniter_state_storage: IgniterStateStorage,
    fuel_tank_state: FuelTankState,
    fuel_tank_state_storage: FuelTankStateStorage,
    igniter_fuel_injector_pressure: f32,
    igniter_gox_injector_pressure: f32,
    igniter_chamber_pressure: f32,
    fuel_tank_pressure: f32,
    ecu_board_temp: f32,
    igniter_throat_temp: f32,
}

pub struct ECUControlPins {
    pub sv1_ctrl: PA12<Output>,
    pub sv2_ctrl: PA11<Output>,
    pub sv3_ctrl: PA10<Output>,
    pub sv4_ctrl: PA9<Output>,
    pub spark_ctrl: PwmChannel<TIM1, 0>,
}

pub fn ecu_update(mut ctx: app::ecu_update::Context) {
    app::ecu_update::spawn_after(10.millis().into()).unwrap();

    let ecu_state = ctx.local.ecu_state;
    let ecu_pins = ctx.local.ecu_control_pins;

    ctx.shared.current_daq_frame.lock(|daq| {
        let apply_sensor_value = |sensor: ECUSensor| -> f32 {
            ecu_state.config
                .sensor_configs[sensor as usize]
                .apply(daq.sensor_values[sensor as usize] as f32)
        };

        ecu_state.igniter_fuel_injector_pressure = apply_sensor_value(ECUSensor::IgniterFuelInjectorPressure);
        ecu_state.igniter_gox_injector_pressure = apply_sensor_value(ECUSensor::IgniterGOxInjectorPressure);
        ecu_state.igniter_chamber_pressure = apply_sensor_value(ECUSensor::IgniterChamberPressure);
        ecu_state.fuel_tank_pressure = apply_sensor_value(ECUSensor::FuelTankPressure);
        ecu_state.igniter_throat_temp = apply_sensor_value(ECUSensor::IgniterThroatTemp);
        ecu_state.ecu_board_temp = raw_board_temp_to_celsius(daq.sensor_values[ECUSensor::ECUBoardTemp as usize]);
    });

    ctx.shared.packet_queue.lock(|packet_queue| {
        while let Some(packet) = packet_queue.dequeue() {
            match packet {
                Packet::FireIgniter => ecu_state.igniter_state_storage.received_fire_igniter_command = true,
                Packet::PressurizeFuelTank => ecu_state.fuel_tank_state_storage.pressurize_command_received = true,
                Packet::DepressurizeFuelTank => ecu_state.fuel_tank_state_storage.depressurize_command_received = true,
                _ => {},
            }
        }
    });

    igniter_fsm::update(ecu_state, ecu_pins, 0.01);
    fuel_tank_fsm::update(ecu_state, ecu_pins, 0.01);

    let telem_frame = ECUTelemetryFrame {
        igniter_state: ecu_state.igniter_state,
        fuel_tank_state: ecu_state.fuel_tank_state,
        sensors: [
            ecu_state.igniter_fuel_injector_pressure,
            ecu_state.igniter_gox_injector_pressure,
            ecu_state.igniter_chamber_pressure,
            ecu_state.fuel_tank_pressure,
            ecu_state.ecu_board_temp,
            ecu_state.igniter_throat_temp,
        ],
        solenoid_valves: [
            ecu_pins.sv1_ctrl.get_state() == PinState::High,
            ecu_pins.sv2_ctrl.get_state() == PinState::High,
            ecu_pins.sv3_ctrl.get_state() == PinState::High,
            ecu_pins.sv4_ctrl.get_state() == PinState::High,
        ],
        sparking: ecu_pins.spark_ctrl.get_duty() != 0,
    };

    app::send_packet::spawn(Packet::ECUTelemetry(telem_frame), NetworkAddress::MissionControl).ok();
}

pub fn ecu_init(ecu_state: &mut ECUState, ecu_pins: &mut ECUControlPins) {
    igniter_fsm::transition_state(ecu_state, ecu_pins, IgniterState::Idle);
    fuel_tank_fsm::transition_state(ecu_state, ecu_pins, FuelTankState::Idle);
}

impl ECUState {
    pub const fn default() -> Self {
        Self {
            config: ECUConfiguration::default(),
            igniter_state: IgniterState::Idle,
            igniter_state_storage: IgniterStateStorage::default(),
            fuel_tank_state: FuelTankState::Idle,
            fuel_tank_state_storage: FuelTankStateStorage::default(),
            igniter_fuel_injector_pressure: 0.0,
            igniter_gox_injector_pressure: 0.0,
            igniter_chamber_pressure: 0.0,
            fuel_tank_pressure: 0.0,
            ecu_board_temp: 0.0,
            igniter_throat_temp: 0.0,
        }
    }
}

fn raw_board_temp_to_celsius(sample: u16) -> f32 {
    // let cal30 = VtempCal30::get().read() as f32;
    // let cal110 = VtempCal110::get().read() as f32;

    sample as f32 // (110.0 - 30.0) * ((sample as f32) - cal30) / (cal110 - cal30) + 30.0
}