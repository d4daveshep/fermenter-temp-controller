from asyncio import StreamReader, StreamWriter
import serial_asyncio
from controller.config import ArduinoConfig
from controller.temperature import TemperatureReading


class ArduinoTempController:
    def __init__(self, config: ArduinoConfig):
        self.config = config

    async def open_serial_connection(self) -> tuple[StreamReader, StreamWriter]:
        reader, writer = await serial_asyncio.open_serial_connection(
            url=self.config.serial_port, baudrate=self.config.baud_rate
        )
        self.serial_port_reader: StreamReader = reader
        self.serial_port_writer: StreamWriter = writer
        return reader, writer

    async def read_temperature(self):  # -> TemperatureReading:
        # TODO: implement
        # return TemperatureReading()
        data: bytes = await self.serial_port_reader.readline()
        return data.decode()
