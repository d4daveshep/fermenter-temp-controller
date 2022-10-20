import random
from time import sleep
import pytest_asyncio
import pytest
import main
import asyncio
from serial.serialutil import SerialException

@pytest.mark.asyncio
async def test_open_serial_connection():

    with pytest.raises(SerialException) as err_info:
        serial_port_reader, serial_port_writer = await main.open_serial_connection()

    pass


@pytest.mark.asyncio
async def test_write_async_serial_string():
    serial_port_reader, serial_port_writer = main.open_serial_connection()

    target_temp = random.randrange(10, 30)

    await main.write_float_to_serial_port(serial_port_writer, target_temp)

    what = False
    assert what


def test_send_and_receive_target_temp_to_serial():
    serial_port = main.get_serial_port()

    # important that we read from serial port before first write
    json = main.read_json_from_serial(serial_port)

    for i in range(10):
        target_temp = random.randrange(10, 30)

        main.write_float_to_serial(serial_port, target_temp)

        sleep(1)

        json = main.read_json_from_serial(serial_port)
        assert json["target"] == target_temp
