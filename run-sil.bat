cd simulation\software-in-loop
maturin develop
cd ..\..
python -m pytest -n 4 simulation/tests
