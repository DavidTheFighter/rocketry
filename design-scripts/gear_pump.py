
displacement_per_revolution = (8 / 1000) # L / rev
test_speed = 2500 # RPM
fluid_density = 1000 # kg/m^3
efficiency = 0.75

test_flow_rate = efficiency * displacement_per_revolution * test_speed / 60.0 # L / s
test_mass_flow_rate = test_flow_rate * fluid_density * 1e-3 # kg / s

print("Test flow rate: {:.4f} L/s".format(test_flow_rate))
print("Test flow rate: {:.4f} kg/s".format(test_mass_flow_rate))

