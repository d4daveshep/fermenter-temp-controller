import pytest
from controller.arduino import ArduinoTempController
from controller.config import ArduinoConfig


@pytest.mark.asyncio
async def test_arduino_open_serial_connection():
    config: ArduinoConfig = ArduinoConfig(serial_port="/dev/ttyACM0", baud_rate=115200)

    arduino = ArduinoTempController(config)
    reader, writer = await arduino.open_serial_connection()
    assert reader is arduino.serial_port_reader
    assert writer is arduino.serial_port_writer
