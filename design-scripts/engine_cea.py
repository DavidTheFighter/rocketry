from rocketcea.cea_obj import CEA_Obj
from props import *
from orifice import *
import numpy as np
import math

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

# --- Design Constants --- #

CHAMBER_PRESSURE = 150.0 # PSI
EXIT_PRESSURE = 10.0 # PSI
MIX_RATIO = 1.2
THRUST = 600 # Newtons

L_STAR = 1.5 # meters
CONVERGENT_HALF_ANGLE = 15.0 * (math.pi / 180.0) # radians

# Calculations

C = CEA_Obj( oxName='LOX', fuelName='Isopropanol70', fac_CR=3.6)

nozzle_expansion_ratio = C.get_eps_at_PcOvPe(Pc=CHAMBER_PRESSURE, PcOvPe=EXIT_PRESSURE, MR=MIX_RATIO)

c_star = C.get_Cstar(Pc=CHAMBER_PRESSURE, MR=MIX_RATIO) * 0.3048
isp = C.get_Isp(Pc=CHAMBER_PRESSURE, MR=MIX_RATIO, eps=nozzle_expansion_ratio)
cf = C.get_PambCf(Pamb=14.7, Pc=CHAMBER_PRESSURE, MR=MIX_RATIO, eps=nozzle_expansion_ratio)[1]

total_mass_flow = THRUST / (c_star * cf)

throat_area = c_star * total_mass_flow / (CHAMBER_PRESSURE * 6894.75729)
throat_diameter = 2.0 * math.sqrt(throat_area / math.pi)

exit_area = throat_area * nozzle_expansion_ratio
exit_diameter = 2.0 * math.sqrt(exit_area / math.pi)

oxid_mass_flow = total_mass_flow * MIX_RATIO / (MIX_RATIO + 1.0)
fuel_mass_flow = total_mass_flow / (MIX_RATIO + 1.0)

chamber_volume = throat_area * L_STAR
chamber_length = calc_chamber_length(throat_diameter)
chamber_diameter = calc_chamber_diameter(throat_diameter, CONVERGENT_HALF_ANGLE, chamber_length, chamber_volume)

contraction_ratio = chamber_diameter / throat_diameter

print("Thrust {:.2f} N".format(THRUST))
print("Cp {:.2f} psi @ {:.2f} MR".format(CHAMBER_PRESSURE, MIX_RATIO))
print("C* {:.2f} m/s, Isp {:.2f} s, Cf {:.2f}".format(c_star, isp, cf))

print("")
print("Total Mass Flow\t\t{:.3f} kg/s ({:.3f} g/s)".format(total_mass_flow, total_mass_flow * 1e3))
print("Oxidizer Mass Flow\t{:.3f} kg/s ({:.3f} g/s)".format(oxid_mass_flow, oxid_mass_flow * 1e3))
print("Fuel Mass Flow\t\t{:.3f} kg/s ({:.3f} g/s)".format(fuel_mass_flow, fuel_mass_flow * 1e3))

print("Throat diameter\t\t{:.3f} m ({:.3f} mm or {:.3f} in)".format(throat_diameter, throat_diameter * 1e3, throat_diameter * 39.3701))
print("Exit diameter\t\t{:.3f} m ({:.3f} mm or {:.3f} in)".format(exit_diameter, exit_diameter * 1e3, exit_diameter * 39.3701))
print("Chamber length\t\t{:.3f} m ({:.3f} cm or {:.3f} in)".format(chamber_length, chamber_length * 1e2, chamber_length * 39.3701))
print("Chamber diameter\t{:.3f} m ({:.3f} cm or {:.3f} in)".format(chamber_diameter, chamber_diameter * 1e2, chamber_diameter * 39.3701))
print("Contraction ratio\t{:.1f}".format(contraction_ratio))

print()
