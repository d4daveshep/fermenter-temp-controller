import pytest
from unittest.mock import AsyncMock, MagicMock, patch
from controller.arduino import ArduinoTempController
from controller.config import ArduinoConfig


@pytest.mark.asyncio
async def test_arduino_open_serial_connection():
    # create mock reader and writer
    mock_reader = AsyncMock()
    mock_writer = MagicMock()

    # configure the reader mock to return specific data
    # TODO: change this to a temperature json string
    mock_reader.readline.return_value = b"mocked response\r\n"

    config: ArduinoConfig = ArduinoConfig(serial_port="/dev/ttyACM0", baud_rate=115200)
    arduino = ArduinoTempController(config)

    # patch the open_serial_connection function
    with patch(
        "serial_asyncio.open_serial_connection", return_value=(mock_reader, mock_writer)
    ):
        # call the function
        reader, writer = await arduino.open_serial_connection()

        assert reader is arduino.serial_port_reader
        assert writer is arduino.serial_port_writer
