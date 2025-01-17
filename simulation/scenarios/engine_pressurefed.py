import sys

import software_in_loop as sil
from simulation.simulation import SimulationBase, build_sim_from_argv
import simulation.config_builder as cb

class EngineSimulation(SimulationBase):
    def __init__(self, sim_config: dict):
        super().__init__(sim_config)

    def initialize(self, project_config: dict, realtime: bool):
        self.project_config = project_config
        self.realtime = realtime

        self.eth_network = sil.SilNetwork([10, 0, 0, 0])

        self.ecu_eth_phy = sil.SilNetworkPhy(self.eth_network)
        self.ecu_eth_iface = sil.SilNetworkIface(self.ecu_eth_phy)

        self.mission_ctrl_eth_phy = sil.SilNetworkPhy(self.eth_network)
        self.mission_ctrl_eth_iface = sil.SilNetworkIface(self.mission_ctrl_eth_phy)

        self.mission_ctrl = sil.MissionControl([self.mission_ctrl_eth_iface], self.realtime)

        self.tank_fuel_pipe = sil.FluidConnection()
        self.engine_fuel_pipe = sil.FluidConnection()
        self.igniter_fuel_pipe = sil.FluidConnection()

        self.tank_oxidizer_pipe = sil.FluidConnection()
        self.engine_oxidizer_pipe = sil.FluidConnection()
        self.igniter_oxidizer_pipe = sil.FluidConnection()

        self.ox_to_fuel_press_pipe = sil.FluidConnection()

        self.fuel_splitter = sil.FluidSplitter(self.tank_fuel_pipe, [self.engine_fuel_pipe, self.igniter_fuel_pipe])
        self.oxidizer_splitter = sil.FluidSplitter(self.tank_oxidizer_pipe, [self.engine_oxidizer_pipe, self.igniter_oxidizer_pipe])

        N2O_VAPOR_PRESSURE_PA = self.project_config["hardwareConfig"]["oxidizerConfig"]["propellantLiquid"]["vaporPressurePa"]
        self.fuel_tank_dynamics = cb.build_fuel_tank(self.project_config["hardwareConfig"], self.tank_fuel_pipe, N2O_VAPOR_PRESSURE_PA, sil.ROOM_TEMP_K)
        self.oxidizer_tank_dynamics = cb.build_oxidizer_tank(self.project_config["hardwareConfig"], self.tank_oxidizer_pipe, N2O_VAPOR_PRESSURE_PA, sil.ROOM_TEMP_K)

        self.igniter_dynamics = cb.build_igniter(self.project_config["hardwareConfig"], self.igniter_fuel_pipe, self.igniter_oxidizer_pipe)
        self.engine_dynamics = cb.build_engine(self.project_config["hardwareConfig"], self.engine_fuel_pipe, self.engine_oxidizer_pipe)

        self.ecu = sil.EcuSil(
            [self.ecu_eth_iface],
            0, # ECU index
            self.project_config["hardwareConfig"]["ecuSensorConfig"],
            self.sim_config["ecu_update_rate"],
            self.fuel_tank_dynamics,
            self.oxidizer_tank_dynamics,
            self.engine_dynamics,
            self.igniter_dynamics,
            None, # self.fuel_pump,
            None, # self.oxidizer_pump,
        )

        self.fuel_tank_dynamics.ullage_inlet = self.ox_to_fuel_press_pipe
        self.oxidizer_tank_dynamics.ullage_outlet = self.ox_to_fuel_press_pipe

        self.dynamics_manager = sil.DynamicsManager()

        self.dynamics_manager.add_dynamics_component(self.ecu)
        self.dynamics_manager.add_dynamics_component(self.mission_ctrl)

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
        self.dynamics_manager.add_dynamics_component(self.ox_to_fuel_press_pipe)

        self.logger = sil.Logger([self.eth_network])
        self.logger.dt = self.sim_config["sim_update_rate"]

        self.ecu.update_ecu_config(self.project_config["softwareConfig"]["ecu0"])

    def advance_timestep(self):
        self.dynamics_manager.update(self.t, self.dt)

        if not self.realtime:
            self.logger.log_common_data()
            self.logger.log_ecu_data(self.ecu)

        self.t += self.dt

        return True

if __name__ == "__main__":
    def engine_app():
        sim_config = {
            "ecu_update_rate": 0.001,
            "sim_update_rate": 0.0005,
            "replay_update_rate": 0.01,
        }

        if len(sys.argv) < 2:
            print("Usage: python engine_pressurefed.py <optional gen script> <config_file> ...")
            return

        ignited = False
        pressurized = False

        def tick_callback(sim: EngineSimulation):
            nonlocal ignited, pressurized

            if not ignited and not pressurized and sim.t > 0.5:
                pressurized = True
                sim.mission_ctrl.fuel_tank.press()
                sim.mission_ctrl.oxidizer_tank.press()

            if not ignited and sim.t > 3.0:
                ignited = True
                sim.mission_ctrl.engine.fire()

            if ignited and sim.t > 20.0:
                return False

            return True

        simulation = EngineSimulation(sim_config)
        build_sim_from_argv(simulation, sys.argv)

        sil.simulate_app(
            simulation,
            None if simulation.realtime else tick_callback,
            simulation.realtime,
        )

    engine_app()
