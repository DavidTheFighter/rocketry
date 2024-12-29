import software_in_loop as sil

def build_fuel_tank(config: dict, outlet, initial_pressure_pa: float, initial_temp_k: float) -> sil.SilTankDynamics:
    return _build_tank_dynamics(config, config['fuelConfig'], outlet, initial_pressure_pa, initial_temp_k)

def build_oxidizer_tank(config: dict, outlet, initial_pressure_pa: float, initial_temp_k: float) -> sil.SilTankDynamics:
    return _build_tank_dynamics(config, config['oxidizerConfig'], outlet, initial_pressure_pa, initial_temp_k)

def _build_tank_dynamics(config: dict, prop_config: dict, outlet, initial_pressure_pa: float, initial_temp_k: float) -> sil.SilTankDynamics:
    if 'pressConfig' in config.keys() and config['pressConfig'] is not None:
        press_config = sil.SilTankPressConfig(
            config['pressConfig']['pressurePa'],
            config['pressConfig']['setPointPa'],
            sil.GasDefinition(
                config['pressConfig']['pressGas']['name'],
                config['pressConfig']['pressGas']['molecularWeightKg'] * 1e3, # kg/mol -> g/mol
                config['pressConfig']['pressGas']['specificHeatRatio'],
            ),
            config['pressConfig']['orificeDiameterMeters'],
            config['pressConfig']['orificeCd'],
            config['pressConfig']['temperatureKelvin'],
        )
    else:
        press_config = None

    return sil.SilTankDynamics(
        press_config,
        # Ullage gas
        sil.GasDefinition(
            prop_config['ullageGas']['name'],
            prop_config['ullageGas']['molecularWeightKg'] * 1e3, # kg/mol -> g/mol
            prop_config['ullageGas']['specificHeatRatio'],
        ),
        # Propellant liquid
        _propellant_definition(prop_config),
        prop_config['ventDiameterMeters'],
        prop_config['ventCd'],
        prop_config['propellantMassKg'],
        initial_pressure_pa,
        initial_temp_k,
        prop_config['tankVolumeMeters3'],
        outlet,
    )

def build_igniter(config: dict, fuel_inlet, oxidizer_inlet) -> sil.SilIgniterDynamics:
    igniter_fuel_injector = sil.InjectorConfig(
        config['igniterConfig']['fuelInjectorDiameterMeters'],
        config['igniterConfig']['fuelInjectorCd'],
        _propellant_definition(config["fuelConfig"]),
    )
    igniter_oxidizer_injector = sil.InjectorConfig(
        config['igniterConfig']['oxidizerInjectorDiameterMeters'],
        config['igniterConfig']['oxidizerInjectorCd'],
        _propellant_definition(config["oxidizerConfig"]),
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
        _propellant_definition(config["fuelConfig"]),
    )
    oxidizer_injector = sil.InjectorConfig(
        config['engineConfig']['oxidizerInjectorDiameterMeters'],
        config['engineConfig']['oxidizerInjectorCd'],
        _propellant_definition(config["oxidizerConfig"]),
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
        (config['fuelPumpConfig']['setPointPsi'] - config['pressConfig']['setPointPsi']) * 6894.76, # Psi -> Pa
    )

def build_oxidizer_pump(config: dict, tank_outlet, pump_outlet) -> sil.SilPumpDynamics:
    return sil.SilPumpDynamics(
        tank_outlet,
        pump_outlet,
        (config['oxidizerPumpConfig']['setPointPsi'] - config['pressConfig']['setPointPsi']) * 6894.76, # Psi -> Pa
    )

def _propellant_definition(prop_config: dict) -> sil.LiquidDefinition:
    return sil.LiquidDefinition(
        prop_config['propellantLiquid']['name'],
        prop_config['propellantLiquid']['densityKgPerM3'],
        prop_config['propellantLiquid']['vaporPressurePa'],
    )

