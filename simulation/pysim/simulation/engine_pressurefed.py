import json

import software_in_loop as sil
from pysim.config import SimConfig
from pysim.replay import SimReplay
from pysim.simulation.simulation import SimulationBase
import pysim.simulation.config_builder as cb

class EnginePumpSimulation(SimulationBase):
    def __init__(self, config: SimConfig, equip_config: dict, loggingQueue=None, log_to_file=False):
        super().__init__(config, loggingQueue, log_to_file)
        self.eth_network = sil.SilNetwork([10, 0, 0, 0])

        self.ecu_eth_phy = sil.SilNetworkPhy(self.eth_network)
        self.ecu_eth_iface = sil.SilNetworkIface(self.ecu_eth_phy)

        self.mission_ctrl_eth_phy = sil.SilNetworkPhy(self.eth_network)
        self.mission_ctrl_eth_iface = sil.SilNetworkIface(self.mission_ctrl_eth_phy)

        self.mission_ctrl = sil.MissionControl([self.mission_ctrl_eth_iface])

        self.tank_fuel_pipe = sil.FluidConnection()
        self.engine_fuel_pipe = sil.FluidConnection()
        self.igniter_fuel_pipe = sil.FluidConnection()

        self.tank_oxidizer_pipe = sil.FluidConnection()
        self.engine_oxidizer_pipe = sil.FluidConnection()
        self.igniter_oxidizer_pipe = sil.FluidConnection()

        self.fuel_splitter = sil.FluidSplitter(self.tank_fuel_pipe, [self.engine_fuel_pipe, self.igniter_fuel_pipe])
        self.oxidizer_splitter = sil.FluidSplitter(self.tank_oxidizer_pipe, [self.engine_oxidizer_pipe, self.igniter_oxidizer_pipe])

        self.fuel_tank_dynamics = cb.build_fuel_tank(equip_config, self.tank_fuel_pipe, sil.ATMOSPHERIC_PRESSURE_PA)
        self.oxidizer_tank_dynamics = cb.build_oxidizer_tank(equip_config, self.tank_oxidizer_pipe, sil.ATMOSPHERIC_PRESSURE_PA)

        self.igniter_dynamics = cb.build_igniter(equip_config, self.igniter_fuel_pipe, self.igniter_oxidizer_pipe)
        self.engine_dynamics = cb.build_engine(equip_config, self.engine_fuel_pipe, self.engine_oxidizer_pipe)

        self.ecu = sil.EcuSil(
            [self.ecu_eth_iface],
            0, # ECU index
            config.ecu_sensor_config,
            self.fuel_tank_dynamics,
            self.oxidizer_tank_dynamics,
            self.engine_dynamics,
            self.igniter_dynamics,
            None, # self.fuel_pump,
            None, # self.oxidizer_pump,
        )

        self.dynamics_manager = sil.DynamicsManager()
        self.dynamics_manager.add_dynamics_component(self.fuel_tank_dynamics)
        self.dynamics_manager.add_dynamics_component(self.oxidizer_tank_dynamics)
        self.dynamics_manager.add_dynamics_component(self.igniter_dynamics)
        self.dynamics_manager.add_dynamics_component(self.engine_dynamics)
        self.dynamics_manager.add_dynamics_component(self.tank_fuel_pipe)
        self.dynamics_manager.add_dynamics_component(self.engine_fuel_pipe)
        self.dynamics_manager.add_dynamics_component(self.igniter_fuel_pipe)
        self.dynamics_manager.add_dynamics_component(self.tank_oxidizer_pipe)
        self.dynamics_manager.add_dynamics_component(self.engine_oxidizer_pipe)
        self.dynamics_manager.add_dynamics_component(self.igniter_oxidizer_pipe)
        self.dynamics_manager.add_dynamics_component(self.fuel_splitter)
        self.dynamics_manager.add_dynamics_component(self.oxidizer_splitter)

        self.logger = sil.Logger([self.eth_network])
        self.logger.dt = self.config.sim_update_rate

        self.ecu.update_ecu_config(self.config.ecu_config)
        self.time_since_last_ecu_update = 0.0

    def advance_timestep(self):
        self.ecu.update_timestamp(self.t)

        self.mission_ctrl.update(self.dt)
        self.dynamics_manager.update(self.dt)

        self.time_since_last_ecu_update += self.dt
        if self.time_since_last_ecu_update >= self.config.ecu_update_rate:
            self.ecu.update(self.config.ecu_update_rate)
            self.time_since_last_ecu_update -= self.config.ecu_update_rate

        self.logger.log_common_data()
        self.logger.log_ecu_data(self.ecu)

        self.t += self.dt

        return True

if __name__ == "__main__":
    def engine_app():
        import sys

        config = SimConfig()
        config.sim_update_rate = 0.0005 # Seconds
        config.main_fuel_pump_pressure_setpoint_pa = 500 * 6894.75729 # PSI to pascals
        config.main_oxidizer_pump_pressure_setpoint_pa = 500 * 6894.75729 # PSI to pascals
        config.ecu_config['engine_config']['use_pumps'] = False

        if len(sys.argv) > 1:
            with open(sys.argv[1], 'r') as f:
                equip_config = json.load(f)
        else:
            print("Usage: python engine_pump.py <config_file>")
            return

        ignited = False
        pressurized = False

        def tick_callback(sim: EnginePumpSimulation):
            nonlocal ignited, pressurized

            if not ignited and not pressurized and sim.t > 0.5:
                pressurized = True
                sim.mission_ctrl.send_set_fuel_tank_packet(0, True)
                sim.mission_ctrl.send_set_oxidizer_tank_packet(0, True)

            if not ignited and sim.t > 3.0:
                ignited = True
                sim.mission_ctrl.send_fire_engine_packet(0)

            if ignited and sim.t > 10.0:
                return False

            return True

        sil.simulate_app_replay(EnginePumpSimulation(config, equip_config), tick_callback)

    engine_app()
