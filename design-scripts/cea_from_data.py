from rocketcea.cea_obj import CEA_Obj
from props import *
from orifice import *

# --- Measured values --- #

CHAMBER_PRESSURE = 65.0 # PSI
EXPANSION_RATIO = 2.5

OXID_ORIFICE_DIAMETER = 0.063 # in
OXID_INJECTOR_PRESSURE = 85 # PSI
OXID_DENSITY = 7.826 # kg/m^3
OXID_ORIFICE_CD = 0.96

FUEL_ORIFICE_DIAMETER = 0.016 # in
FUEL_INJECTOR_PRESSURE = 200 # PSI
FUEL_DENSITY = 855.0 # kg/m^3
FUEL_ORIFICE_CD = 0.937

OXID_KAPPA = 1.4
PIPE_DIAMETER = 0.25 # in

# --- Calculations --- #

C = CEA_Obj( oxName='GOX', fuelName='Ethanol75')

oxid_mass_flow = orifice_compressible(
    OXID_ORIFICE_DIAMETER * 0.0254,
    PIPE_DIAMETER * 0.0254,
    OXID_ORIFICE_CD,
    OXID_INJECTOR_PRESSURE * 6894.75729,
    CHAMBER_PRESSURE * 6894.75729,
    OXID_DENSITY,
    kappa=OXID_KAPPA,
)

fuel_mass_flow = orifice_incompressible(
    FUEL_ORIFICE_DIAMETER * 0.0254,
    PIPE_DIAMETER * 0.0254,
    FUEL_ORIFICE_CD,
    FUEL_INJECTOR_PRESSURE * 6894.75729,
    CHAMBER_PRESSURE * 6894.75729,
    FUEL_DENSITY,
)

mix_ratio = oxid_mass_flow / fuel_mass_flow
c_star = C.get_Cstar(Pc=CHAMBER_PRESSURE, MR=mix_ratio) * 0.3048
isp = C.get_Isp(Pc=CHAMBER_PRESSURE, MR=mix_ratio, eps=EXPANSION_RATIO)
cf = C.get_PambCf(Pc=CHAMBER_PRESSURE, MR=mix_ratio, eps=EXPANSION_RATIO)[1]

thrust = (oxid_mass_flow + fuel_mass_flow) * c_star * cf

print("C*\t{:.2f} m/s".format(c_star))
print("Isp\t{:.2f} s".format(isp))
print("Cf\t{:.2f}".format(cf))

print("")
print("Thrust\t\t\t{:.2f} N".format(thrust))
print("Oxid mass flow\t\t{:.2f} kg/s ({:.3f} g/s)".format(oxid_mass_flow, oxid_mass_flow * 1e3))
print("Fuel mass flow\t\t{:.2f} kg/s ({:.3f} g/s)".format(fuel_mass_flow, fuel_mass_flow * 1e3))