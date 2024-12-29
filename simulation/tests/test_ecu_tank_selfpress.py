import software_in_loop as sil
import pytest

from simulation.pysim.scenarios.simulation import SimulationBase
import simulation.pysim.scenarios.config_builder as cb

def tanks_pressurized(sim: SimulationBase):
    fuel_threshold_pa = sim.ecu_config["fuel_tank_config"]["press_min_threshold_pa"]
    oxidizer_threshold_pa = sim.ecu_config["oxidizer_tank_config"]["press_min_threshold_pa"]

    return sim.fuel_tank_dynamics.tank_pressure_pa > fuel_threshold_pa \
        and sim.oxidizer_tank_dynamics.tank_pressure_pa > oxidizer_threshold_pa

def tanks_depressurized(sim: SimulationBase):
    fuel_threshold_pa = sim.ecu_config["fuel_tank_config"]["press_min_threshold_pa"]
    oxidizer_threshold_pa = sim.ecu_config["oxidizer_tank_config"]["press_min_threshold_pa"]

    return sim.fuel_tank_dynamics.tank_pressure_pa < fuel_threshold_pa \
        and sim.oxidizer_tank_dynamics.tank_pressure_pa < oxidizer_threshold_pa

def test_tanks_init_state(tank_sim):
    tank_sim.advance_timestep()
    SELF_PRESS_PRESSURE_PA = tank_sim.project_config["hardwareConfig"]["oxidizerConfig"]["propellantLiquid"]["vaporPressurePa"]

    def assert_idle_state(tank_sim: TankOnlySimulation):
        assert tank_sim.ecu['igniter_state'] == 'Idle'
        assert tank_sim.ecu['binary_valves']['FuelPressValve'] == False
        assert tank_sim.ecu['binary_valves']['OxidizerPressValve'] == False
        # Note: Intentionally ignoring vent valve states for now
        assert tank_sim.fuel_tank_dynamics.tank_pressure_pa > SELF_PRESS_PRESSURE_PA * 0.95
        assert tank_sim.oxidizer_tank_dynamics.tank_pressure_pa > SELF_PRESS_PRESSURE_PA * 0.95
        # Test sim physics to verify pressure doesn't increase for no reason
        assert tank_sim.fuel_tank_dynamics.tank_pressure_pa < SELF_PRESS_PRESSURE_PA * 1.05
        assert tank_sim.oxidizer_tank_dynamics.tank_pressure_pa < SELF_PRESS_PRESSURE_PA * 1.05

    tank_sim.simulate_assert(assert_idle_state, 5.0)

def test_fuel_tank_depress_and_repress(tank_sim):
    tank_sim.advance_timestep()

    tank_sim.mission_ctrl.send_set_fuel_tank_packet(0, False)

    last_pressure_pa = tank_sim.fuel_tank_dynamics.tank_pressure_pa * 1.01
    def assert_pressure_decreasing(tank_sim: TankOnlySimulation):
        nonlocal last_pressure_pa
        decreasing = tank_sim.fuel_tank_dynamics.tank_pressure_pa < last_pressure_pa
        last_pressure_pa = tank_sim.fuel_tank_dynamics.tank_pressure_pa
        return decreasing

    tank_sim.simulate_assert(assert_pressure_decreasing, 1.0)

    assert tank_sim.ecu['fuel_tank_state'] == 'Venting'
    assert tank_sim.ecu['oxidizer_tank_state'] == 'Idle'
    assert tank_sim.ecu['binary_valves']['FuelVentValve'] == True
    assert tank_sim.ecu['binary_valves']['OxidizerVentValve'] == False

    tank_sim.mission_ctrl.send_set_fuel_tank_packet(0, True)

    def assert_pressure_increasing(tank_sim: TankOnlySimulation):
        nonlocal last_pressure_pa
        increasing = tank_sim.fuel_tank_dynamics.tank_pressure_pa > last_pressure_pa
        last_pressure_pa = tank_sim.fuel_tank_dynamics.tank_pressure_pa
        return increasing

    tank_sim.simulate_assert(assert_pressure_increasing, 1.0)

    assert tank_sim.ecu['fuel_tank_state'] == 'Pressurized'
    assert tank_sim.ecu['oxidizer_tank_state'] == 'Idle'
    assert tank_sim.ecu['binary_valves']['FuelVentValve'] == False

def test_oxidizer_tank_depress_and_repress(tank_sim):
    tank_sim.advance_timestep()

    tank_sim.mission_ctrl.send_set_oxidizer_tank_packet(0, False)

    last_pressure_pa = tank_sim.oxidizer_tank_dynamics.tank_pressure_pa * 1.01
    def assert_pressure_decreasing(tank_sim: TankOnlySimulation):
        nonlocal last_pressure_pa
        decreasing = tank_sim.oxidizer_tank_dynamics.tank_pressure_pa < last_pressure_pa
        last_pressure_pa = tank_sim.oxidizer_tank_dynamics.tank_pressure_pa
        return decreasing

    tank_sim.simulate_assert(assert_pressure_decreasing, 1.0)

    assert tank_sim.ecu['fuel_tank_state'] == 'Idle'
    assert tank_sim.ecu['oxidizer_tank_state'] == 'Venting'
    assert tank_sim.ecu['binary_valves']['FuelVentValve'] == False
    assert tank_sim.ecu['binary_valves']['OxidizerVentValve'] == True

    tank_sim.mission_ctrl.send_set_oxidizer_tank_packet(0, True)

    def assert_pressure_increasing(tank_sim: TankOnlySimulation):
        nonlocal last_pressure_pa
        increasing = tank_sim.oxidizer_tank_dynamics.tank_pressure_pa > last_pressure_pa
        last_pressure_pa = tank_sim.oxidizer_tank_dynamics.tank_pressure_pa
        return increasing

    tank_sim.simulate_assert(assert_pressure_increasing, 1.0)

    assert tank_sim.ecu['fuel_tank_state'] == 'Idle'
    assert tank_sim.ecu['oxidizer_tank_state'] == 'Pressurized'
    assert tank_sim.ecu['binary_valves']['OxidizerVentValve'] == False

@pytest.fixture
def tank_sim(project_config):
    sim_config = {
        "ecu_update_rate": 0.001,
        "sim_update_rate": 0.0005,
    }

    simulation = TankOnlySimulation(sim_config)
    simulation.initialize(project_config, False) # False for no realtime

    return simulation

class TankOnlySimulation(SimulationBase):
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
        self.tank_oxidizer_pipe = sil.FluidConnection()
        self.ox_to_fuel_press_pipe = sil.FluidConnection()

        N2O_VAPOR_PRESSURE_PA = self.project_config["hardwareConfig"]["oxidizerConfig"]["propellantLiquid"]["vaporPressurePa"]
        self.fuel_tank_dynamics = cb.build_fuel_tank(self.project_config["hardwareConfig"], self.tank_fuel_pipe, N2O_VAPOR_PRESSURE_PA, sil.ROOM_TEMP_K)
        self.oxidizer_tank_dynamics = cb.build_oxidizer_tank(self.project_config["hardwareConfig"], self.tank_oxidizer_pipe, N2O_VAPOR_PRESSURE_PA, sil.ROOM_TEMP_K)

        self.ecu = sil.EcuSil(
            [self.ecu_eth_iface],
            0, # ECU index
            self.project_config["hardwareConfig"]["ecuSensorConfig"],
            self.sim_config["ecu_update_rate"],
            self.fuel_tank_dynamics,
            self.oxidizer_tank_dynamics,
            None, # self.engine_dynamics,
            None, # self.igniter_dynamics,
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
        self.dynamics_manager.add_dynamics_component(self.tank_fuel_pipe)
        self.dynamics_manager.add_dynamics_component(self.tank_oxidizer_pipe)
        self.dynamics_manager.add_dynamics_component(self.ox_to_fuel_press_pipe)

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

@pytest.fixture
def project_config(generic_ecu_sensor_config):
    config = {}

    config["hardwareConfig"] = {
        "pressConfig": None,
        "fuelConfig": {
            "ventDiameterMeters": 0.01,
            "ventCd": 0.65,
            "tankVolumeMeters3": 0.005,
            "propellantMassKg": 4.0,
            "propellantLiquid": {
                "name": "75% IPA",
                "densityKgPerM3": 846.0,
                "vaporPressurePa": 4.1,
            },
            "ullageGas": {
                "name": "N2O",
                "molecularWeightKg": 0.04401,
                "specificHeatRatio": 0.875,
            },
        },
        "oxidizerConfig": {
            "ventDiameterMeters": 0.1,
            "ventCd": 0.65,
            "tankVolumeMeters3": 0.03,
            "propellantMassKg": 10.0,
            "propellantLiquid": {
                "name": "N2O",
                "densityKgPerM3": 1220.0,
                "vaporPressurePa": 5137000.0,
            },
            "ullageGas": {
                "name": "N2O",
                "molecularWeightKg": 0.04401,
                "specificHeatRatio": 0.875,
            },
        },
        "igniterConfig": None,
        "fuelPumpConfig": None,
        "oxidizerPumpConfig": None,
        "engineConfig": None,
        "ecuSensorConfig": generic_ecu_sensor_config,
    }

    config["softwareConfig"] = {
        "ecu0": {
            "engine_config": None,
            "igniter_config": None,
            "fuel_tank_config": {
                'press_valve': None,
                'vent_valve': "FuelVentValve",
                'fill_valve': "FuelFillValve",
                'press_min_threshold_pa': 500.0 * 6894.76, # PSI to Pascals
                'press_max_threshold_pa': 900.0 * 6894.76, # PSI to Pascals
            },
            "oxidizer_tank_config": {
                'press_valve': None,
                'vent_valve': "OxidizerVentValve",
                'fill_valve': "OxidizerFillValve",
                'press_min_threshold_pa': 500.0 * 6894.76, # PSI to Pascals
                'press_max_threshold_pa': 900.0 * 6894.76, # PSI to Pascals
            },
            "telemetry_rate_s": 0.02,
        }
    }

    return config
