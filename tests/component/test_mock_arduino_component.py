import asyncio
import json
from typing import Any

import pytest
import serial_asyncio


@pytest.mark.asyncio
async def test_mock_arduino_serial_output_generator(mock_serial_connection):
    """
    Test using the mock_serial_connection that returns as reader,writer tuple.
    The reader returns output from the mock_arduino_output_generator on each readline() call.
    By referencing the mock_serial_connection fixture we will get our patched serial connection.
    """
    # open serial connection (uses patched connection)
    reader, writer = await serial_asyncio.open_serial_connection(
        url="/dev/ttyUSB0",
        baudrate=9600,
    )

    for _ in range(60):
        try:
            data: bytes = await reader.readline()
            decoded_str: str = data.decode().strip()
            if decoded_str:
                arduino_json: dict[str, Any] = json.loads(decoded_str)
                print(f"arduino output: {decoded_str}")
                assert "json-size" in arduino_json
                assert arduino_json["json-size"] == 12345
                # await asyncio.sleep(0.1)
        except Exception as e:
            print(f"Error reading from mock arduino serial connection: {e}")
            # await asyncio.sleep(0.5)

    assert mock_serial_connection["reader"].readline.call_count == 60


@pytest.mark.asyncio
async def test_mock_arduino_serial_write_target_temp(mock_serial_connection):
    """
    Test the mock serial connection can accept a target temperature and
    update the target temperature returned on the next output
    """
    assert False, "implement this test"
