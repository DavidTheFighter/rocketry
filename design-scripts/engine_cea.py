from rocketcea.cea_obj import CEA_Obj
from props import *
from orifice import *
import numpy as np

# --- Design Constants --- #

CHAMBER_PRESSURE = 150.0 # PSI
EXIT_PRESSURE = 10.0 # PSI
MIX_RATIO = 1.0
THRUST = 500 # Newtons

# Calculations

C = CEA_Obj( oxName='GOX', fuelName='Isopropanol70')

nozzle_expansion_ratio = C.get_eps_at_PcOvPe(Pc=CHAMBER_PRESSURE, PcOvPe=EXIT_PRESSURE, MR=MIX_RATIO)

c_star = C.get_Cstar(Pc=CHAMBER_PRESSURE, MR=MIX_RATIO) * 0.3048
isp = C.get_Isp(Pc=CHAMBER_PRESSURE, MR=MIX_RATIO, eps=nozzle_expansion_ratio)
cf = C.get_PambCf(Pamb=14.7, Pc=CHAMBER_PRESSURE, MR=MIX_RATIO, eps=nozzle_expansion_ratio)[1]

total_mass_flow = THRUST / (c_star * cf)

throat_area = c_star * total_mass_flow / (CHAMBER_PRESSURE * 6894.75729)
throat_diameter = (throat_area / 3.1415926)**(0.5)

exit_area = throat_area * nozzle_expansion_ratio
exit_diameter = (exit_area / 3.1415926)**(0.5)

oxid_mass_flow = total_mass_flow * MIX_RATIO / (MIX_RATIO + 1.0)
fuel_mass_flow = total_mass_flow / (MIX_RATIO + 1.0)

print("Thrust {:.2f} N".format(THRUST))
print("Cp {:.2f} psi @ {:.2f} MR".format(CHAMBER_PRESSURE, MIX_RATIO))
print("C* {:.2f} m/s, Isp {:.2f} s, Cf {:.2f}".format(c_star, isp, cf))

print("")
print("Total Mass Flow\t\t{:.3f} kg/s ({:.3f} g/s)".format(total_mass_flow, total_mass_flow * 1e3))
print("Oxidizer Mass Flow\t{:.3f} kg/s ({:.3f} g/s)".format(oxid_mass_flow, oxid_mass_flow * 1e3))
print("Fuel Mass Flow\t\t{:.3f} kg/s ({:.3f} g/s)".format(fuel_mass_flow, fuel_mass_flow * 1e3))

print("Throat diameter\t\t{:.3f} m ({:.3f} mm)".format(throat_diameter, throat_diameter * 1e3))
print("Exit diameter\t\t{:.3f} m ({:.3f} mm)".format(exit_diameter, exit_diameter * 1e3))

print()