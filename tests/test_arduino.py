import json
import asyncio
from typing import Any, Generator
import pytest
from unittest.mock import AsyncMock, MagicMock, patch, _patch
from controller.arduino import ArduinoTempController
from controller.config import ArduinoConfig


@pytest.fixture
def mock_reader() -> AsyncMock:
    reader = AsyncMock()

    # configure the reader mock to return specific data
    # TODO: change this to a temperature json string
    reader.readline.return_value = b"mocked response\r\n"

    return reader


@pytest.fixture
def mock_serial_connection() -> Generator[Any, Any, Any]:
    """Fixture to create a mock serial connection that return JSON strings"""
    mock_reader: AsyncMock = AsyncMock()
    mock_writer: MagicMock = MagicMock()

    # Store the data that will be returned on each readline call
    json_data: list[dict[str, Any]] = [
        {"temperature": 22.5, "humidity": 45, "timestamp": "2025-05-19T10:00:00"},
        {"temperature": 22.7, "humidity": 46, "timestamp": "2025-05-19T10:00:10"},
        {"temperature": 22.9, "humidity": 47, "timestamp": "2025-05-19T10:00:20"},
    ]

    # Convert data to JSON strings with newline (as would be received over serial)
    json_strings: list[bytes] = [
        (json.dumps(data) + "\r\n").encode() for data in json_data
    ]

    # configure readline to return different values on successive calls
    mock_reader.readline.side_effect = json_strings

    # create a patch for serial_asyncio.open_serial_connection
    patcher: _patch[AsyncMock] = patch(
        "serial_asyncio.open_serial_connection", new=AsyncMock()
    )
    mock_open: AsyncMock = patcher.start()
    mock_open.return_value = (mock_reader, mock_writer)

    # TODO: add mock sleep

    # return everything needed for testing
    yield {
        "reader": mock_reader,
        "writer": mock_writer,
        "json_data": json_data,
        "patcher": patcher,
    }

    # clean up the patcher after the test is complete
    patcher.stop()


@pytest.mark.asyncio
async def test_arduino_open_serial_connection(mock_reader: AsyncMock):
    # create mock reader and writer
    # mock_reader = AsyncMock()
    mock_writer = MagicMock()

    config: ArduinoConfig = ArduinoConfig(serial_port="/dev/ttyACM0", baud_rate=115200)
    arduino = ArduinoTempController(config)

    # patch the open_serial_connection function
    with patch(
        "serial_asyncio.open_serial_connection", return_value=(mock_reader, mock_writer)
    ) as mock_open:
        # call the function
        reader, writer = await arduino.open_serial_connection()

        assert reader is mock_reader
        assert writer is mock_writer
        assert reader is arduino.serial_port_reader
        assert writer is arduino.serial_port_writer

        reading: str = await arduino.read_temperature()
        assert reading == "mocked response"


@pytest.mark.asyncio
async def test_monitor_serial_port(mock_serial_connection: dict):
    """ 
    Test the monitoring of the serial port using mocked serial connection
    """

    config: ArduinoConfig = ArduinoConfig(serial_port="/dev/ttyACM0", baud_rate=115200)
    arduino = ArduinoTempController(config)

    received_data: list[dict[str, Any]] = [] # to hold the json data we receive

    # create a callback function to add the received data
    def data_callback(data: dict[str, Any]):
        received_data.append(data)

    # run the monitoring function
    monitor_task = asyncio.create_task(
        arduino.monitor_serial_port(callback=data_callback)
    )

    # sleep for a bit to allow the tests to run
    await asyncio.sleep(1.0)

    # stop the monitoring function
    monitor_task.cancel()

    # check that the data was received
    assert len(received_data) == 3
    assert received_data == mock_serial_connection["json_data"]
