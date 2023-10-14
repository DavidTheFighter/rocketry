test_upstream = 90 # PSI
test_downstream = 14.7 # PSI

desired_downstream = 300 # PSI
desired_massflow = 0.22 # kg/s

fluid_density = 1000 # kg/m^3

C = 0.95
orifice_diameter = 0.063 # inches

# --- Calculations but in metric --- #

import math

def flow_power(kg_per_s, delta_pressure):
    return kg_per_s * 9.81 * delta_pressure * 0.00010199773339984

test_upstream = test_upstream * 6894.75729 # Pa
test_downstream = test_downstream * 6894.75729 # Pa

desired_downstream = desired_downstream * 6894.75729 # Pa

orifice_diameter = orifice_diameter * 0.0254 # m
orifice_area = math.pi * math.pow(orifice_diameter, 2.0) / 4.0 # m^2

test_flow_rate = C * orifice_area * math.sqrt(2.0 * fluid_density * (test_upstream - test_downstream))

test_flow_power = flow_power(test_flow_rate, test_upstream - test_downstream) # test_flow_rate * 9.81 * (test_upstream - test_downstream) * 0.00010199773339984

print("Test flow rate: {:.4f} kg/s".format(test_flow_rate))
print("Test flow power: {:.4f} W".format(test_flow_power))

desired_upstream = test_flow_power / (desired_massflow * 9.81 * 0.00010199773339984)

print("Equivalent upstream pressure: {:.2f} PSI".format(desired_upstream / 6894.75729))

# desired_upstream = (test_upstream - test_downstream) * math.pow(desired_massflow / test_flow_rate, 2.0) + desired_downstream

# print("Test flow rate: {:.4f} kg/s".format(test_flow_rate))
# print("Equivalent upstream pressure: {:.2f} PSI".format(desired_upstream / 6894.75729))

# desired_flow_rate = C * math.pow(orifice_diameter, 2.0) * math.sqrt(2.0 * fluid_density * (desired_upstream - desired_downstream))

# print("Equivalent flow rate: {:.4f} kg/s".format(desired_flow_rate))