import sys

import software_in_loop as sil
from simulation.pysim.scenarios.simulation import SimulationBase, build_sim_from_argv
import simulation.pysim.scenarios.config_builder as cb

class IgniterSimulation(SimulationBase):
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

        self.fuel_splitter = sil.FluidSplitter(self.tank_fuel_pipe, [self.engine_fuel_pipe, self.igniter_fuel_pipe])
        self.oxidizer_splitter = sil.FluidSplitter(self.tank_oxidizer_pipe, [self.engine_oxidizer_pipe, self.igniter_oxidizer_pipe])

        N2O_VAPOR_PRESSURE_PA = self.project_config["hardwareConfig"]["oxidizerConfig"]["propellantLiquid"]["vaporPressurePa"]
        self.fuel_tank_dynamics = cb.build_fuel_tank(self.project_config["hardwareConfig"], self.tank_fuel_pipe, N2O_VAPOR_PRESSURE_PA, sil.ROOM_TEMP_K)
        self.oxidizer_tank_dynamics = cb.build_oxidizer_tank(self.project_config["hardwareConfig"], self.tank_oxidizer_pipe, N2O_VAPOR_PRESSURE_PA, sil.ROOM_TEMP_K)

        self.igniter_dynamics = cb.build_igniter(self.project_config["hardwareConfig"], self.igniter_fuel_pipe, self.igniter_oxidizer_pipe)

        self.ecu = sil.EcuSil(
            [self.ecu_eth_iface],
            0, # ECU index
            self.project_config["hardwareConfig"]["ecuSensorConfig"],
            self.sim_config["ecu_update_rate"],
            self.fuel_tank_dynamics,
            self.oxidizer_tank_dynamics,
            None,
            self.igniter_dynamics,
            None,
            None,
        )

        self.dynamics_manager = sil.DynamicsManager()

        self.dynamics_manager.add_dynamics_component(self.ecu)
        self.dynamics_manager.add_dynamics_component(self.mission_ctrl)

        self.dynamics_manager.add_dynamics_component(self.fuel_tank_dynamics)
        self.dynamics_manager.add_dynamics_component(self.oxidizer_tank_dynamics)
        self.dynamics_manager.add_dynamics_component(self.igniter_dynamics)
        self.dynamics_manager.add_dynamics_component(self.tank_fuel_pipe)
        self.dynamics_manager.add_dynamics_component(self.engine_fuel_pipe)
        self.dynamics_manager.add_dynamics_component(self.igniter_fuel_pipe)
        self.dynamics_manager.add_dynamics_component(self.tank_oxidizer_pipe)
        self.dynamics_manager.add_dynamics_component(self.engine_oxidizer_pipe)
        self.dynamics_manager.add_dynamics_component(self.igniter_oxidizer_pipe)
        self.dynamics_manager.add_dynamics_component(self.fuel_splitter)
        self.dynamics_manager.add_dynamics_component(self.oxidizer_splitter)

        self.logger = sil.Logger([self.eth_network])
        self.logger.dt = self.sim_config["sim_update_rate"]

        self.ecu_config = self.project_config["softwareConfig"]["ecu0"]
        self.ecu.update_ecu_config(self.ecu_config)

    def advance_timestep(self):
        self.dynamics_manager.update(self.t, self.dt)

        if not self.realtime:
            self.logger.log_common_data()
            self.logger.log_ecu_data(self.ecu)

        self.t += self.dt

        return True

if __name__ == "__main__":
    def igniter_app():
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

        simulation = IgniterSimulation(sim_config)
        build_sim_from_argv(simulation, sys.argv)

        sil.simulate_app(
            simulation,
            None if simulation.realtime else tick_callback,
            simulation.realtime,
        )

    igniter_app()
