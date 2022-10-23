import asyncio
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

    async def _read_line_from_serial(self) -> str:
        line = await self._serial_port_reader.readline()
        print(f"read {line} from serial")
        return str(line, 'utf-8')

    async def _read_temps_from_serial_and_write_to_database(self):

        while True:
            json_read = await self._read_line_from_serial()

            point = self.temperature_database.create_point_from_fermenter_json(json_read)
            self.temperature_database.write_temperature_record(point)

    def run(self):

        loop = asyncio.get_event_loop()

        try:
            self._serial_port_reader, self._serial_port_writer = loop.run_until_complete(
                self._open_serial_connection(self.config.serial_port))

            discard_first_record = loop.run_until_complete(self._read_line_from_serial())

            reader_task = loop.create_task(self._read_temps_from_serial_and_write_to_database())

            loop.run_forever()

        except KeyboardInterrupt:
            print("Stopping loop")
            loop.stop()
        finally:
            print("Closing loop")
            loop.close()


if __name__ == "__main__":
    filename = "./test_valid_config_file.txt"

    controller = TempController(filename)

    controller.run()
