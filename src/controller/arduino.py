import asyncio
from asyncio import StreamReader, StreamWriter
import json
from typing import Callable, Any
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

    # async def read_temperature(self):  # -> TemperatureReading:
    #     data: bytes = await self.serial_port_reader.readline()
    #     return data.decode(encoding="utf-8").strip()
    #     # return TemperatureReading()

    async def monitor_serial_port(
        # FIXME: change callable function to accept a string parameter not a dict
        self,
        callback: Callable[[dict[str, Any]], None] | None,
    ):
        """
        Monitor the serial port calling the callback function when a line of data is received
        """
        await self.open_serial_connection()

        try:
            while True:
                data: bytes = await self.serial_port_reader.readline()
                if not data:
                    # FIXME: use logging instead of print
                    print("ERROR: No data received from serial port")
                    break

                try:
                    json_string: str = data.decode(encoding="utf-8").strip()
                    try:
                        json_data: dict[str, Any] = json.loads(json_string)
                        if callback:
                            callback(json_data)
                    except json.JSONDecodeError:
                        # FIXME: use logging instead of print
                        print(f"Invalid JSON: {json_string}")
                except UnicodeDecodeError:
                    # FIXME: use logging instead of print
                    print(f"Can't decode bytes: {data}")

                await asyncio.sleep(0)
        finally:
            self.serial_port_writer.close()

    async def write_to_serial_port(self, string_to_write: str) -> None:
        """
        Encode a string to bytes and write to the serial port
        """
        self.serial_port_writer.write(string_to_write.encode())
        await asyncio.sleep(0)
