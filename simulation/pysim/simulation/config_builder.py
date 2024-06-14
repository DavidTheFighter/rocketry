import software_in_loop as sil

def build_fuel_tank(config: dict, outlet, initial_pressure_pa: float) -> sil.SilTankDynamics:
    return _build_tank_dynamics(config, config['fuelConfig'], outlet, initial_pressure_pa)

def build_oxidizer_tank(config: dict, outlet, initial_pressure_pa: float) -> sil.SilTankDynamics:
    return _build_tank_dynamics(config, config['oxidizerConfig'], outlet, initial_pressure_pa)

def _build_tank_dynamics(config: dict, prop_config: dict, outlet, initial_pressure_pa: float) -> sil.SilTankDynamics:
    feed_config = sil.SilTankFeedConfig(
        config['feedConfig']['pressurePsi'] * 6894.76, # Psi -> Pa
        config['feedConfig']['setPointPsi'] * 6894.76, # Psi -> Pa
        sil.GasDefinition(
            config['feedConfig']['feedGas']['name'],
            config['feedConfig']['feedGas']['molecularWeightKg'] * 1e3, # kg/mol -> g/mol
            config['feedConfig']['feedGas']['specificHeatRatio'],
        ),
        config['feedConfig']['orificeDiameterMeters'],
        config['feedConfig']['orificeCd'],
        config['feedConfig']['temperatureKelvin'],
    )

    return sil.SilTankDynamics(
        feed_config,
        prop_config['ventDiameterMeters'],
        prop_config['ventCd'],
        initial_pressure_pa,
        prop_config['tankVolumeMeters3'],
        outlet,
    )

def build_igniter(config: dict, fuel_inlet, oxidizer_inlet) -> sil.SilIgniterDynamics:
    igniter_fuel_injector = sil.InjectorConfig(
        config['igniterConfig']['fuelInjectorDiameterMeters'],
        config['igniterConfig']['fuelInjectorCd'],
        _fuel_definition(config),
    )
    igniter_oxidizer_injector = sil.InjectorConfig(
        config['igniterConfig']['oxidizerInjectorDiameterMeters'],
        config['igniterConfig']['oxidizerInjectorCd'],
        _oxidizer_definition(config),
    )

    combustion_data_tmp = sil.CombustionData(
        0.55, # Mixture ratio
        0.03, # Combustion product kg/mol
        1.3, # Combustion product specific heat ratio
        2000, # Chamber temperature in K
    )

    return sil.SilIgniterDynamics(
        fuel_inlet,
        oxidizer_inlet,
        igniter_fuel_injector,
        igniter_oxidizer_injector,
        combustion_data_tmp,
        config['igniterConfig']['throatDiameterMeters'],
    )

def build_engine(config: dict, fuel_inlet, oxidizer_inlet) -> sil.SilEngineDynamics:
    fuel_injector = sil.InjectorConfig(
        config['engineConfig']['fuelInjectorDiameterMeters'],
        config['engineConfig']['fuelInjectorCd'],
        _fuel_definition(config),
    )
    oxidizer_injector = sil.InjectorConfig(
        config['engineConfig']['oxidizerInjectorDiameterMeters'],
        config['engineConfig']['oxidizerInjectorCd'],
        _oxidizer_definition(config),
    )

    combustion_data_tmp = sil.CombustionData(
        0.55, # Mixture ratio
        0.03, # Combustion product kg/mol
        1.3, # Combustion product specific heat ratio
        2000, # Chamber temperature in K
    )

    return sil.SilEngineDynamics(
        fuel_inlet,
        oxidizer_inlet,
        fuel_injector,
        oxidizer_injector,
        combustion_data_tmp,
        config['engineConfig']['throatDiameterMeters'],
    )

def build_fuel_pump(config: dict, tank_outlet, pump_outlet) -> sil.SilPumpDynamics:
    return sil.SilPumpDynamics(
        tank_outlet,
        pump_outlet,
        (config['fuelPumpConfig']['setPointPsi'] - config['feedConfig']['setPointPsi']) * 6894.76, # Psi -> Pa
    )

def build_oxidizer_pump(config: dict, tank_outlet, pump_outlet) -> sil.SilPumpDynamics:
    return sil.SilPumpDynamics(
        tank_outlet,
        pump_outlet,
        (config['oxidizerPumpConfig']['setPointPsi'] - config['feedConfig']['setPointPsi']) * 6894.76, # Psi -> Pa
    )

def _fuel_definition(config: dict) -> sil.LiquidDefinition:
    return sil.LiquidDefinition(
        config['fuelConfig']['fuelLiquid']['name'],
        config['fuelConfig']['fuelLiquid']['densityKgPerM3'],
    )

def _oxidizer_definition(config: dict) -> sil.LiquidDefinition:
    return sil.LiquidDefinition(
        config['oxidizerConfig']['oxidizerLiquid']['name'],
        config['oxidizerConfig']['oxidizerLiquid']['densityKgPerM3'],
    )
