ethanol75 = """
fuel C2H5OH(L)  C 2   H 6   O 1     wt%=75.0
h,cal=-66370.0     t(k)=298.15   rho,g/cc=.789
fuel   water(L)  H 2 O 1   wt%=25.0
h,cal=-68308.0     t(k)=298.15   rho,g/cc = 0.9998
"""

isopropanol70 = """
fuel C3H8O-2propanol C 3 H 8 O 1    wt%=70.0
h,cal=-65133.     t(k)=298.15   rho=0.786
fuel   water  H 2 O 1   wt%=30.0
h,cal=-68308.0     t(k)=298.15   rho,g/cc = 0.9998
"""

isopropanol75 = """
fuel C3H8O-2propanol C 3 H 8 O 1    wt%=75.0
h,cal=-65133.     t(k)=298.15   rho=0.786
fuel   water  H 2 O 1   wt%=25.0
h,cal=-68308.0     t(k)=298.15   rho,g/cc = 0.9998
"""

isopropanol90 = """
fuel C3H8O-2propanol C 3 H 8 O 1    wt%=90.0
h,cal=-65133.     t(k)=298.15   rho=0.786
fuel   water  H 2 O 1   wt%=10.0
h,cal=-68308.0     t(k)=298.15   rho,g/cc = 0.9998
"""

from rocketcea.cea_obj import add_new_fuel

add_new_fuel("Ethanol75", ethanol75)
add_new_fuel("Isopropanol70", isopropanol70)
add_new_fuel("Isopropanol75", isopropanol75)
add_new_fuel("Isopropanol90", isopropanol90)