import time, math

import software_in_loop as sil
from pysim.config import SimConfig
from pysim.replay import SimReplay
from pysim.glue import SilGlue

class IgniterSimulation:
    def __init__(self, config: SimConfig, loggingQueue=None, log_to_file=False):
        self.config = config
        self.glue = SilGlue()

        self.eth_network = sil.SilNetwork([10, 0, 0, 0])

        self.ecu_eth_phy = sil.SilNetworkPhy(self.eth_network)
        self.ecu_eth_iface = sil.SilNetworkIface(self.ecu_eth_phy)

        self.mission_ctrl_eth_phy = sil.SilNetworkPhy(self.eth_network)
        self.mission_ctrl_eth_iface = sil.SilNetworkIface(self.mission_ctrl_eth_phy)

        self.glue.ecu = self.ecu = sil.EcuSil([self.ecu_eth_iface])
        self.glue.mission_ctrl = self.mission_ctrl = sil.MissionControl([self.mission_ctrl_eth_iface])

        self.feed_config = sil.SilTankFeedConfig(
            2000 * 6894.76, # Feed pressure in Pa
            200 * 6894.76, # Setpoint pressure in Pa
            sil.GasDefinition('GN2', 28.02, 1.039),
            0.002, # Feed orifice diameter in m
            0.6, # Feed orifice coefficient of discharge
            293.15, # Feed temperature in K
        )
        self.glue.fuel_tank_dynamics = self.fuel_tank_dynamics = sil.SilTankDynamics(
            self.feed_config,
            0.001, # Vent orifice diameter in m
            0.65, # Vent orifice coefficient of discharge
            14.7 * 6894.76, # Initial tank pressure in Pa
            0.005, # Tank volume in m^3
        )
        self.glue.oxidizer_tank_dynamics = self.oxidizer_tank_dynamics = sil.SilTankDynamics(
            self.feed_config,
            0.001, # Vent orifice diameter in m
            0.65, # Vent orifice coefficient of discharge
            14.7 * 6894.76, # Initial tank pressure in Pa
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

        self.glue.igniter_dynamics = self.igniter_dynamics = sil.SilIgniterDynamics(
            self.igniter_fuel_injector,
            self.igniter_oxidizer_injector,
            self.combustion_data_tmp,
            0.004, # Throat diameter in m
        )

        self.logger = sil.Logger([self.eth_network])
        self.logger.dt = self.config.sim_update_rate
        self.logging = loggingQueue
        self.log_to_file = log_to_file

        self.dt = self.config.sim_update_rate
        self.t = 0.0
        self.pressurized = False
        self.ignited = False

    def simulate_until_done(self):
        start_time = time.time()

        while self.advance_timestep():
            pass

        print("Simulation took {:.2f} s".format(time.time() - start_time))

        if self.log_to_file:
            self.logger.dump_to_file()

    def advance_timestep(self):
        self.mission_ctrl.update_timestep(self.dt)
        self.ecu.update_timestamp(self.t)

        if math.fmod(self.t, config.ecu_pressure_sensor_rate) <= self.dt + config.sim_update_rate * 0.1:
            self.ecu.update_fuel_tank_pressure(self.fuel_tank_dynamics.tank_pressure_pa)

        if math.fmod(self.t, config.ecu_pressure_sensor_rate) <= self.dt + config.sim_update_rate * 0.1:
            self.ecu.update_oxidizer_tank_pressure(self.oxidizer_tank_dynamics.tank_pressure_pa)

        if math.fmod(self.t, config.ecu_pressure_sensor_rate) <= self.dt + config.sim_update_rate * 0.1:
            self.ecu.update_igniter_chamber_pressure(self.igniter_dynamics.chamber_pressure_pa)

        self.glue.update(self.dt)
        self.fuel_tank_dynamics.update(self.dt)
        self.oxidizer_tank_dynamics.update(self.dt)
        self.igniter_dynamics.update(self.dt)
        self.ecu.update(self.dt)

        self.logger.log_common_data()
        self.logger.log_ecu_data(self.ecu)

        self.t += self.dt

        if not self.pressurized and self.t > 0.5:
            self.pressurized = True
            self.mission_ctrl.send_set_fuel_tank_packet(0, True)
            self.mission_ctrl.send_set_oxidizer_tank_packet(0, True)

        if not self.ignited and self.t > 3.0:
            self.ignited = True
            self.mission_ctrl.send_fire_igniter_packet(0)

        if self.ignited and self.t > 10.0:
            return False

        return True

    def replay(self):
        replay = SimReplay(self.config, self.logger)
        replay.replay(self.logging)

if __name__ == "__main__":
    from pysim.app import simulate_app

    config = SimConfig()

    simulate_app(config, IgniterSimulation)
