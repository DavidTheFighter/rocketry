import time, sys

import software_in_loop as sil
from pysim.config import SimConfig
from pysim.replay import SimReplay
from pysim.simulation.simulation import SimulationBase

class IgniterPumpSimulation(SimulationBase):
    def __init__(self, config: SimConfig, loggingQueue=None, log_to_file=False, realtime=False):
        super().__init__(config, loggingQueue, log_to_file)
        self.eth_network = sil.SilNetwork([10, 0, 0, 0])

        self.ecu_eth_phy = sil.SilNetworkPhy(self.eth_network)
        self.ecu_eth_iface = sil.SilNetworkIface(self.ecu_eth_phy)

        self.mission_ctrl_eth_phy = sil.SilNetworkPhy(self.eth_network)
        self.mission_ctrl_eth_iface = sil.SilNetworkIface(self.mission_ctrl_eth_phy)

        self.mission_ctrl = sil.MissionControl([self.mission_ctrl_eth_iface], realtime)

        self.tank_fuel_pipe = sil.FluidConnection()
        self.pump_fuel_pipe = sil.FluidConnection()
        self.tank_oxidizer_pipe = sil.FluidConnection()
        self.pump_oxidizer_pipe = sil.FluidConnection()

        self.feed_config = sil.SilTankFeedConfig(
            2000 * 6894.76, # Feed pressure in Pa
            50.0 * 6894.76, # Setpoint pressure in Pa
            sil.GasDefinition('GN2', 28.02, 1.039),
            0.004, # Feed orifice diameter in m
            0.6, # Feed orifice coefficient of discharge
            293.15, # Feed temperature in K
        )
        self.fuel_tank_dynamics = sil.SilTankDynamics(
            self.feed_config,
            self.config.ecu_tank_vent_diamter_m, # Vent orifice diameter in m
            0.65, # Vent orifice coefficient of discharge
            sil.ATMOSPHERIC_PRESSURE_PA, # Initial tank pressure in Pa
            0.005, # Tank volume in m^3
            self.tank_fuel_pipe,
        )
        self.oxidizer_tank_dynamics = sil.SilTankDynamics(
            self.feed_config,
            self.config.ecu_tank_vent_diamter_m, # Vent orifice diameter in m
            0.65, # Vent orifice coefficient of discharge
            sil.ATMOSPHERIC_PRESSURE_PA, # Initial tank pressure in Pa
            0.01, # Tank volume in m^3
            self.tank_oxidizer_pipe,
        )

        self.igniter_fuel_injector = sil.InjectorConfig(
            0.016 * 0.0254, # Injector orifice diameter in m
            0.75, # Injector orifice coefficient of discharge
            sil.LiquidDefinition('75% IPA', 846),
        )
        self.igniter_oxidizer_injector = sil.InjectorConfig(
            0.016 * 0.0254, # Injector orifice diameter in m
            0.75, # Injector orifice coefficient of discharge
            sil.LiquidDefinition('LOX', 1141),
        )
        self.combustion_data_tmp = sil.CombustionData(
            0.55, # Mixture ratio
            0.03, # Combustion product kg/mol
            1.3, # Combustion product specific heat ratio
            2000, # Chamber temperature in K
        )
        self.igniter_dynamics = sil.SilIgniterDynamics(
            self.pump_fuel_pipe,
            self.pump_oxidizer_pipe,
            self.igniter_fuel_injector,
            self.igniter_oxidizer_injector,
            self.combustion_data_tmp,
            0.004, # Throat diameter in m
        )

        self.fuel_pump = sil.SilPumpDynamics(
            self.tank_fuel_pipe,
            self.pump_fuel_pipe,
            config.main_fuel_pump_pressure_setpoint_pa - config.ecu_tank_pressure_set_point_pa, # Pump pressure rise in Pa
        )

        self.oxidizer_pump = sil.SilPumpDynamics(
            self.tank_oxidizer_pipe,
            self.pump_oxidizer_pipe,
            config.main_oxidizer_pump_pressure_setpoint_pa - config.ecu_tank_pressure_set_point_pa, # Pump pressure rise in Pa
        )

        self.ecu = sil.EcuSil(
            [self.ecu_eth_iface],
            0,
            config.ecu_sensor_config,
            self.fuel_tank_dynamics,
            self.oxidizer_tank_dynamics,
            None,
            self.igniter_dynamics,
            self.fuel_pump,
            self.oxidizer_pump,
        )

        self.dynamics_manager = sil.DynamicsManager()
        self.dynamics_manager.add_dynamics_component(self.fuel_tank_dynamics)
        self.dynamics_manager.add_dynamics_component(self.oxidizer_tank_dynamics)
        self.dynamics_manager.add_dynamics_component(self.igniter_dynamics)
        self.dynamics_manager.add_dynamics_component(self.tank_fuel_pipe)
        self.dynamics_manager.add_dynamics_component(self.pump_fuel_pipe)
        self.dynamics_manager.add_dynamics_component(self.tank_oxidizer_pipe)
        self.dynamics_manager.add_dynamics_component(self.pump_oxidizer_pipe)
        self.dynamics_manager.add_dynamics_component(self.fuel_pump)
        self.dynamics_manager.add_dynamics_component(self.oxidizer_pump)

        self.logger = sil.Logger([self.eth_network])
        self.logger.dt = self.config.sim_update_rate

        self.ecu.update_ecu_config(self.config.ecu_config)
        self.time_since_last_ecu_update = 0.0

        self.realtime = realtime
        self.time_since_last_realtime_wait = 0
        self.last_realtime_wait_time = time.time()

    def advance_timestep(self):
        start_time = time.time()

        self.ecu.update_timestamp(self.t)

        self.mission_ctrl.update(self.dt)
        self.dynamics_manager.update(self.dt)

        self.time_since_last_ecu_update += self.dt
        if self.time_since_last_ecu_update >= self.config.ecu_update_rate:
            self.ecu.update(self.config.ecu_update_rate)
            self.time_since_last_ecu_update -= self.config.ecu_update_rate

        if not self.realtime:
            self.logger.log_common_data()
            self.logger.log_ecu_data(self.ecu)
        elif self.time_since_last_realtime_wait >= 0.01:
            while time.time() - self.last_realtime_wait_time < 0.01:
                pass
            self.last_realtime_wait_time = time.time()
            self.time_since_last_realtime_wait -= 0.01

        self.t += self.dt
        self.time_since_last_realtime_wait += self.dt

        return True

if __name__ == "__main__":
    realtime = len(sys.argv) > 1 and sys.argv[1] == "-r"

    def igniter_app():
        config = SimConfig()
        config.sim_update_rate = 0.001 # Seconds

        ignited = False
        pressurized = False
        pumped = False

        def tick_callback(sim: IgniterPumpSimulation):
            nonlocal ignited, pressurized, pumped

            if not ignited and not pressurized and sim.t > 0.5:
                pressurized = True
                sim.mission_ctrl.send_set_fuel_tank_packet(0, True)
                sim.mission_ctrl.send_set_oxidizer_tank_packet(0, True)

            if not pumped and pressurized and sim.t > 1.5:
                pumped = True
                sim.mission_ctrl.send_set_fuel_pump_packet(0, True)
                sim.mission_ctrl.send_set_oxidizer_pump_packet(0, True)

            if not ignited and sim.t > 3.0:
                ignited = True
                sim.mission_ctrl.send_fire_igniter_packet(0)

            if pressurized and sim.t > 6.0:
                pressurized = False
                sim.mission_ctrl.send_set_fuel_tank_packet(0, False)
                sim.mission_ctrl.send_set_oxidizer_tank_packet(0, False)
                sim.mission_ctrl.send_set_fuel_pump_packet(0, False)
                sim.mission_ctrl.send_set_oxidizer_pump_packet(0, False)

            if ignited and sim.t > 10.0:
                return False

            return True

        sil.simulate_app_replay(IgniterPumpSimulation(config, realtime=realtime), None if realtime else tick_callback)

    igniter_app()
