from rocketcea.cea_obj import CEA_Obj
from props import *
from orifice import *
import numpy as np

# --- Design Constants --- #

CHAMBER_PRESSURE = 42.0 # PSI
EXIT_PRESSURE = 10.0 # PSI
MIX_RATIO = 0.55
THRUST = 5 # Newtons

OXID_INJECTOR_PRESSURE = 85 # PSI
OXID_DENSITY = 7.826 # kg/m^3
OXID_KAPPA = 1.4
OXID_ORIFICE_CD = 0.96

FUEL_INJECTOR_PRESSURE = 200 # PSI
FUEL_DENSITY = 789.0 # kg/m^3
FUEL_ORIFICE_CD = 0.937

INJECTOR_PIPE_DIAMETER = 0.25 # in

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

oxid_injector_diameter = orifice_compressible_diameter(
    oxid_mass_flow,
    INJECTOR_PIPE_DIAMETER * 0.0254,
    OXID_ORIFICE_CD,
    OXID_INJECTOR_PRESSURE * 6894.75729,
    CHAMBER_PRESSURE * 6894.75729,
    OXID_DENSITY,
    kappa=OXID_KAPPA
)

fuel_injector_diameter = orifice_incompressible_diameter(
    fuel_mass_flow,
    INJECTOR_PIPE_DIAMETER * 0.0254,
    FUEL_ORIFICE_CD,
    FUEL_INJECTOR_PRESSURE * 6894.75729,
    CHAMBER_PRESSURE * 6894.75729,
    FUEL_DENSITY
)

print("Thrust {:.2f} N".format(THRUST))
print("Cp {:.2f} psi @ {:.2f} MR".format(CHAMBER_PRESSURE, MIX_RATIO))
print("C* {:.2f} m/s, Isp {:.2f} s, Cf {:.2f}".format(c_star, isp, cf))

# print("")
# print("Total Mass Flow\t\t{:.3f} kg/s ({:.3f} g/s)".format(total_mass_flow, total_mass_flow * 1e3))
# print("Oxidizer Mass Flow\t{:.3f} kg/s ({:.3f} g/s)".format(oxid_mass_flow, oxid_mass_flow * 1e3))
# print("Fuel Mass Flow\t\t{:.3f} kg/s ({:.3f} g/s)".format(fuel_mass_flow, fuel_mass_flow * 1e3))

print("Throat diameter\t\t{:.3f} m ({:.3f} mm)".format(throat_diameter, throat_diameter * 1e3))
# print("Exit diameter\t\t{:.3f} m ({:.3f} mm)".format(exit_diameter, exit_diameter * 1e3))

print("Fuel Pressure\t\t{:.2f} psi".format(orifice_incompressible_pressure(fuel_mass_flow, 0.016 * 0.0254, INJECTOR_PIPE_DIAMETER * 0.0254, FUEL_ORIFICE_CD, CHAMBER_PRESSURE * 6894.75729, FUEL_DENSITY) / 6894.75729))
print("GOx Pressure\t\t{:.2f} psi".format(orifice_compressible_pressure(oxid_mass_flow, 0.063 * 0.0254, INJECTOR_PIPE_DIAMETER * 0.0254, OXID_ORIFICE_CD, CHAMBER_PRESSURE * 6894.75729, OXID_DENSITY * (CHAMBER_PRESSURE / OXID_INJECTOR_PRESSURE), OXID_KAPPA) / 6894.75729))

print()