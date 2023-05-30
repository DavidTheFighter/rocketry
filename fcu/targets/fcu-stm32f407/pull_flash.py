import serial

with open('flash_chip.bin', 'wb') as f:
    f.truncate(0)

with serial.Serial('/dev/ttyUSB0', 115200, timeout=1, parity=serial.PARITY_NONE, stopbits=serial.STOPBITS_ONE) as ser:
    print(ser.name)
    ser.write([0x42])

    bytes_received = 0
    while bytes_received < 256 * 256:
        data = ser.read(512)
        data = [int(data[i:i+2], base=16) for i in range(0, 512, 2)]
        bytes_received += len(data)

        print(bytes_received, "- data =", data)
        with open('flash_chip.bin', 'ab') as f:
            f.write(bytes(data))