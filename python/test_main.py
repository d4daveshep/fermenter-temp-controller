import random
from os.path import exists

import pytest
from serial.serialutil import SerialException

import config
import main


@pytest.fixture
def valid_config():
    filename = "./test_valid_config_file.txt"
    assert exists(filename)

    controller_config = config.ControllerConfig(filename)
    assert controller_config

    return controller_config


@pytest.mark.asyncio
async def test_open_serial_connection(valid_config):
    serial_port_reader, serial_port_writer = await main.open_serial_connection(valid_config.serial_port)


@pytest.mark.asyncio
async def test_open_serial_connection_with_bad_port_url():
    with pytest.raises(SerialException) as err_info:
        serial_port_reader, serial_port_writer = await main.open_serial_connection("/dev/blah_blah")


@pytest.mark.asyncio
async def test_write_async_serial_string(valid_config):
    serial_port_reader, serial_port_writer = await main.open_serial_connection(valid_config.serial_port)

    target_temp = random.randrange(10, 30)

    await main.write_float_to_serial_port(serial_port_writer, target_temp)


@pytest.mark.asyncio
async def test_read_line_from_serial(valid_config):
    serial_port_reader, serial_port_writer = await main.open_serial_connection(valid_config.serial_port)

    read_string = await main.read_line_from_serial(serial_port_reader)

    assert read_string


@pytest.mark.asyncio
async def test_serial_async_write_and_read(valid_config):
    serial_port_reader, serial_port_writer = await main.open_serial_connection(valid_config.serial_port)

    target_temp_written = random.randrange(10, 30)
    await main.write_float_to_serial_port(serial_port_writer, target_temp_written)
    read_string = await main.read_line_from_serial(serial_port_reader)

    target_temp_read = main.get_target_temp_from_serial_string(read_string)

    assert target_temp_written == target_temp_read

# def test_send_and_receive_target_temp_to_serial():
#     serial_port = main.get_serial_port()
#
#     # important that we read from serial port before first write
#     json = main.read_json_from_serial(serial_port)
#
#     for i in range(1):
#         target_temp = random.randrange(10, 30)
#
#         main.write_float_to_serial(serial_port, target_temp)
#
#         sleep(1)
#
#         json = main.read_json_from_serial(serial_port)
#         assert json["target"] == target_temp
