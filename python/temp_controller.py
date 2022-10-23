from asyncio import StreamReader, StreamWriter

import serial_asyncio

from config import ControllerConfig
from temperature_database import TemperatureDatabase


class TempController:
    def __init__(self, config_filename: str):
        self.config = ControllerConfig(config_filename)
        self.temperature_database = TemperatureDatabase(self.config)

        # self.serial_port_reader, self.serial_port_writer = await self._open_serial_connectionpen_serial_connection(self.config.serial_port)

    async def _open_serial_connection(self, port_url: str) -> (StreamReader, StreamWriter):
        return await serial_asyncio.open_serial_connection(url=port_url, baudrate=115200)


if __name__ == "__main__":
    pass
