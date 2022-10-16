import random
from time import sleep

import main


def test_send_and_received_target_temp_to_serial():
    serial_port = main.get_serial_port()

    # important that we read from serial port before first write
    json = main.read_json_from_serial(serial_port)

    for i in range(10):
        target_temp = random.randrange(10, 30)

        main.write_float_to_serial(serial_port, target_temp)

        sleep(1)

        json = main.read_json_from_serial(serial_port)
        assert json["target"] == target_temp
