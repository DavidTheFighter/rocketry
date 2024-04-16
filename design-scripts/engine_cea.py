from rocketcea.cea_obj import CEA_Obj
from props import *
from orifice import *
import numpy as np
import math
from dataclasses import dataclass

@dataclass
class EngineDesign:
    oxName: str
    fuelName: str
    contraction_ratio: float
    chamber_pressure: float
    injector_pressure: float
    exit_pressure: float
    mix_ratio: float
    thrust: float
    l_star: float
    convergent_half_angle: float

ENDREGA_1KN = EngineDesign(
    oxName='LOX',
    fuelName='Isopropanol70',
    contraction_ratio=3.6,
    chamber_pressure=300.0, # PSI
    injector_pressure=500.0, # PSI
    exit_pressure=10.0, # PSI
    mix_ratio=1.2,  # O/F
    thrust=2000.0,  # Newtons
    l_star=1.5, # meters
    convergent_half_angle=15.0 # degrees
)

CURRENT_DESIGN = ENDREGA_1KN

def main(design: EngineDesign):
    C = CEA_Obj(oxName=design.oxName, fuelName=design.fuelName, fac_CR=design.contraction_ratio)

    nozzle_expansion_ratio = C.get_eps_at_PcOvPe(Pc=design.chamber_pressure, PcOvPe=design.exit_pressure, MR=design.mix_ratio)

    c_star = C.get_Cstar(Pc=design.chamber_pressure, MR=design.mix_ratio) * 0.3048
    isp = C.get_Isp(Pc=design.chamber_pressure, MR=design.mix_ratio, eps=nozzle_expansion_ratio)
    cf = C.get_PambCf(Pamb=14.7, Pc=design.chamber_pressure, MR=design.mix_ratio, eps=nozzle_expansion_ratio)[1]

    total_mass_flow = design.thrust / (c_star * cf)

    throat_area = c_star * total_mass_flow / (design.chamber_pressure * 6894.75729)
    throat_diameter = 2.0 * math.sqrt(throat_area / math.pi)

    exit_area = throat_area * nozzle_expansion_ratio
    exit_diameter = 2.0 * math.sqrt(exit_area / math.pi)

    oxid_mass_flow = total_mass_flow * design.mix_ratio / (design.mix_ratio + 1.0)
    fuel_mass_flow = total_mass_flow / (design.mix_ratio + 1.0)

    chamber_volume = throat_area * design.l_star
    chamber_length = calc_chamber_length(throat_diameter)
    chamber_diameter = calc_chamber_diameter(throat_diameter, design.convergent_half_angle * (math.pi / 180.0), chamber_length, chamber_volume)

    contraction_ratio = chamber_diameter / throat_diameter

    example_cd = 0.75
    fuel_orifice_area = fuel_mass_flow / (example_cd * math.sqrt(2.0 * (design.injector_pressure - design.chamber_pressure) * 6894.76 * 846))
    oxid_orifice_area = oxid_mass_flow / (example_cd * math.sqrt(2.0 * (design.injector_pressure - design.chamber_pressure) * 6894.76 * 1141))
    single_fuel_orifice_diameter = 2.0 * math.sqrt(fuel_orifice_area / math.pi)
    single_oxid_orifice_diameter = 2.0 * math.sqrt(oxid_orifice_area / math.pi)

    print("Thrust {:.2f} N".format(design.thrust))
    print("Cp {:.2f} psi @ {:.2f} MR".format(design.chamber_pressure, design.mix_ratio))
    print("C* {:.2f} m/s, Isp {:.2f} s, Cf {:.2f}".format(c_star, isp, cf))

    print("")
    print("Total Mass Flow\t\t{:.3f} kg/s ({:.3f} g/s)".format(total_mass_flow, total_mass_flow * 1e3))
    print("Oxidizer Mass Flow\t{:.3f} kg/s ({:.3f} g/s)".format(oxid_mass_flow, oxid_mass_flow * 1e3))
    print("Fuel Mass Flow\t\t{:.3f} kg/s ({:.3f} g/s)".format(fuel_mass_flow, fuel_mass_flow * 1e3))

    print("")
    print("Oxidizer volume flow\t{:.3f} L/s ({:.4f} ft3/s)".format(oxid_mass_flow * 1000 / 1141.0, oxid_mass_flow * 1000 / 1141.0 * 0.0353147))
    print("Fuel volume flow\t{:.3f} L/s ({:.4f} ft3/s)".format(fuel_mass_flow * 1000 / 786.0, fuel_mass_flow * 1000 / 786.0 * 0.0353147))

    print("")
    print("Throat diameter\t\t{:.3f} m ({:.3f} mm or {:.3f} in)".format(throat_diameter, throat_diameter * 1e3, throat_diameter * 39.3701))
    print("Exit diameter\t\t{:.3f} m ({:.3f} mm or {:.3f} in)".format(exit_diameter, exit_diameter * 1e3, exit_diameter * 39.3701))
    print("Chamber length\t\t{:.3f} m ({:.3f} cm or {:.3f} in)".format(chamber_length, chamber_length * 1e2, chamber_length * 39.3701))
    print("Chamber diameter\t{:.3f} m ({:.3f} cm or {:.3f} in)".format(chamber_diameter, chamber_diameter * 1e2, chamber_diameter * 39.3701))
    print("Contraction ratio\t{:.1f}".format(contraction_ratio))

    print()

    print("Minimum oxidizer fluid power\t{:.3f} W".format(oxid_mass_flow * 9.81 * 0.00010199773339984 * design.injector_pressure * 6894.75729))
    print("Minimum fuel fluid power\t{:.3f} W".format(fuel_mass_flow * 9.81 * 0.00010199773339984 * design.injector_pressure * 6894.75729))

    print()

    print("Example injector CD\t{:.2f}".format(example_cd))
    print("Fuel orifice area\t{:.3f} mm2 ({:.3f} mm single orifice)".format(fuel_orifice_area * 1e6, single_fuel_orifice_diameter * 1e3))
    print("Oxidizer orifice area\t{:.3f} mm2  ({:.3f} mm single orifice)".format(oxid_orifice_area * 1e6, single_oxid_orifice_diameter * 1e3))

def calc_chamber_length(throat_diameter):
    # From http://www.braeunig.us/space/propuls.htm, Figure 1.7

    dt = throat_diameter * 100.0 # In cm
    chamber_length = math.exp(0.029 * math.pow(math.log(dt), 2.0) + 0.47 * math.log(dt) + 1.94) # In cm

    return chamber_length / 100.0

def calc_chamber_diameter(throat_diameter, half_angle, chamber_length, chamber_volume, epsilon=1e-5):
    old_chamber_diameter = throat_diameter * 5.0
    epsilon = old_chamber_diameter * epsilon

    while True:
        numerator = math.pow(throat_diameter, 3.0) + (24.0 / math.pi) * math.tan(half_angle) * chamber_volume
        demoninator = old_chamber_diameter + 6 * math.tan(half_angle) * chamber_length

        chamber_diameter = math.sqrt(numerator / demoninator)

        if abs(chamber_diameter - old_chamber_diameter) < epsilon:
            return chamber_diameter

        old_chamber_diameter = chamber_diameter

if __name__ == "__main__":
    main(CURRENT_DESIGN)
