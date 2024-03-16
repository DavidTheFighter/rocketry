import time, math

import software_in_loop as sil
from pysim.config import SimConfig
from pysim.replay import SimReplay
from pysim.simulation.simulation import SimulationBase

class IgniterSimulation(SimulationBase):
    def __init__(self, config: SimConfig, loggingQueue=None, log_to_file=False):
        super().__init__(config, loggingQueue, log_to_file)
        self.glue = sil.SilGlue()

        self.eth_network = sil.SilNetwork([10, 0, 0, 0])

        self.ecu_eth_phy = sil.SilNetworkPhy(self.eth_network)
        self.ecu_eth_iface = sil.SilNetworkIface(self.ecu_eth_phy)

        self.mission_ctrl_eth_phy = sil.SilNetworkPhy(self.eth_network)
        self.mission_ctrl_eth_iface = sil.SilNetworkIface(self.mission_ctrl_eth_phy)

        self.ecu = sil.EcuSil([self.ecu_eth_iface], 0)
        self.mission_ctrl = sil.MissionControl([self.mission_ctrl_eth_iface])

        self.feed_config = sil.SilTankFeedConfig(
            2000 * 6894.76, # Feed pressure in Pa
            self.config.ecu_tank_pressure_set_point_pa, # Setpoint pressure in Pa
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
        )
        self.oxidizer_tank_dynamics = sil.SilTankDynamics(
            self.feed_config,
            self.config.ecu_tank_vent_diamter_m, # Vent orifice diameter in m
            0.65, # Vent orifice coefficient of discharge
            sil.ATMOSPHERIC_PRESSURE_PA, # Initial tank pressure in Pa
            0.01, # Tank volume in m^3
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
            self.igniter_fuel_injector,
            self.igniter_oxidizer_injector,
            self.combustion_data_tmp,
            0.004, # Throat diameter in m
        )

        self.glue.set_from_self(self)

        self.logger = sil.Logger([self.eth_network])
        self.logger.dt = self.config.sim_update_rate

        self.ecu.update_ecu_config(self.config.ecu_config)
        self.test_t = 0.0

    def advance_timestep(self):
        self.ecu.update_timestamp(self.t)

        if math.fmod(self.t, self.config.ecu_pressure_sensor_rate) <= self.dt + self.config.sim_update_rate * 0.1:
            self.ecu.update_fuel_tank_pressure(self.fuel_tank_dynamics.tank_pressure_pa)

        if math.fmod(self.t, self.config.ecu_pressure_sensor_rate) <= self.dt + self.config.sim_update_rate * 0.1:
            self.ecu.update_oxidizer_tank_pressure(self.oxidizer_tank_dynamics.tank_pressure_pa)

        if math.fmod(self.t, self.config.ecu_pressure_sensor_rate) <= self.dt + self.config.sim_update_rate * 0.1:
            self.ecu.update_igniter_chamber_pressure(self.igniter_dynamics.chamber_pressure_pa)

        # TODO these pressure values need to be dependent on if the valve is open or not
        if math.fmod(self.t, self.config.ecu_pressure_sensor_rate) <= self.dt + self.config.sim_update_rate * 0.1:
            self.ecu.update_igniter_fuel_injector_pressure(self.igniter_dynamics.fuel_pressure_pa)

        if math.fmod(self.t, self.config.ecu_pressure_sensor_rate) <= self.dt + self.config.sim_update_rate * 0.1:
            self.ecu.update_igniter_oxidizer_injector_pressure(self.igniter_dynamics.oxidizer_pressure_pa)

        self.glue.update(self.dt)

        self.mission_ctrl.update(self.dt)
        self.fuel_tank_dynamics.update(self.dt)
        self.oxidizer_tank_dynamics.update(self.dt)
        self.igniter_dynamics.update(self.dt)

        if math.fmod(self.t, self.config.ecu_update_rate) <= self.dt + self.config.sim_update_rate * 0.1:
            self.ecu.update(self.config.ecu_update_rate)

        self.logger.log_common_data()
        self.logger.log_ecu_data(self.ecu)

        self.t += self.dt

        return True

if __name__ == "__main__":
    def igniter_app():
        config = SimConfig()
        config.sim_update_rate = 0.0005 # Seconds

        ignited = False
        pressurized = False

        def tick_callback(sim: IgniterSimulation):
            nonlocal ignited, pressurized

            if not ignited and not pressurized and sim.t > 0.5:
                pressurized = True
                sim.mission_ctrl.send_set_fuel_tank_packet(0, True)
                sim.mission_ctrl.send_set_oxidizer_tank_packet(0, True)

            if not ignited and sim.t > 2.0:
                ignited = True
                sim.mission_ctrl.send_fire_igniter_packet(0)

            if pressurized and sim.t > 6.0:
                pressurized = False
                sim.mission_ctrl.send_set_fuel_tank_packet(0, False)
                sim.mission_ctrl.send_set_oxidizer_tank_packet(0, False)

            if ignited and sim.t > 10.0:
                return False

            return True

        sil.simulate_app_replay(IgniterSimulation(config), tick_callback)

    igniter_app()
