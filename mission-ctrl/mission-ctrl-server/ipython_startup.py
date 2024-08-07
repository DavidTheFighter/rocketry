import mission_ctrl_api

_command_handler = mission_ctrl_api.CommandHandler("ws://127.0.0.1:8000/packet-proxy")

fuel_tank = mission_ctrl_api.Tank("FuelMain", 0, _command_handler)
ox_tank = mission_ctrl_api.Tank("OxidizerMain", 0, _command_handler)

fuel_pump = mission_ctrl_api.Pump("FuelMain", 0, _command_handler)
ox_pump = mission_ctrl_api.Pump("OxidizerMain", 0, _command_handler)

igniter = mission_ctrl_api.Igniter(0, _command_handler)

class _Routines:
    def __init__(self):
        pass

    def press_tanks(self):
        fuel_tank.press()
        ox_tank.press()

    def depress_tanks(self):
        fuel_tank.depress()
        ox_tank.depress()

    def idle_tanks(self):
        fuel_tank.idle()
        ox_tank.idle()

    def fire_igniter_pumped(self):
        import time
        import threading

        def _fire_igniter():
            fuel_pump.full()
            ox_pump.full()
            time.sleep(1.5)
            igniter.fire()
            time.sleep(2.5)
            fuel_pump.off()
            ox_pump.off()

        t = threading.Thread(target=_fire_igniter)
        t.start()

routines = _Routines()

print('Mission control API: [fuel_tank, ox_tank, fuel_pump, ox_pump, igniter, routines]')
