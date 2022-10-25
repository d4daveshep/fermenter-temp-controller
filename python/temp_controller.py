import asyncio
import json
from asyncio import StreamReader, StreamWriter

import serial_asyncio

from config import ControllerConfig
from temperature_database import TemperatureDatabase


class TempController:
    def __init__(self, config_filename: str):
        self.config = ControllerConfig(config_filename)
        self.temperature_database = TemperatureDatabase(self.config)
        self.current_target_temp = 0.0

        self._serial_port_writer = None
        self._serial_port_reader = None

        # self.serial_port_reader, self.serial_port_writer = await self._open_serial_connectionpen_serial_connection(self.config.serial_port)

    async def open_serial_connection(self, port_url: str) -> (StreamReader, StreamWriter):
        return await serial_asyncio.open_serial_connection(url=port_url, baudrate=115200)

    async def read_line_from_serial(self) -> str:
        line = await self._serial_port_reader.readline()
        print(f"read {line} from serial")
        return str(line, 'utf-8')

    async def write_float_to_serial_port(self, float_num: float) -> None:
        string_to_write = '<' + str(float_num) + '>'
        print(f"writing {string_to_write} to serial")
        self._serial_port_writer.write(string_to_write.encode())
        await asyncio.sleep(0)

    def fix_json_values(self, json_dict: dict) -> dict:

        for key, value in json_dict.items():
            # fix and remove some specific fields
            if key == "json-size":  # don't bother writing this to DB
                # del json_dict[key]
                continue
            if type(value) == int:  # covert any ints to floats (e.g. target)
                json_dict[key] = float(value)

        return json_dict

    async def read_temps_from_serial_and_write_to_database(self):

        while True:
            try:

                json_read = await self.read_line_from_serial()
                json_dict = json.loads(json_read)

                data_dict = self.fix_json_values(json_dict)

                # store some key data fields
                self.current_target_temp = data_dict["target"]

                point = self.temperature_database.create_point_from_fermenter_json(json_dict)
                self.temperature_database.write_temperature_record(point)

                await asyncio.sleep(0)

            except Exception as err_info:
                print(err_info)

    async def write_target_temp_to_serial_port_if_updated(self):

        while True:
            # check if target_temp needs updating

            if self.config.target_temp != self.current_target_temp:
                print(f"target temp needs updating to {self.config.target_temp}")
                await self.write_float_to_serial_port(self.config.target_temp)

            await asyncio.sleep(1)

    def run(self):

        loop = asyncio.get_event_loop()

        try:
            self._serial_port_reader, self._serial_port_writer = loop.run_until_complete(
                self.open_serial_connection(self.config.serial_port))

            discard_first_record = loop.run_until_complete(self.read_line_from_serial())

            reader_task = loop.create_task(self.read_temps_from_serial_and_write_to_database())

            writer_task = loop.create_task(self.write_target_temp_to_serial_port_if_updated())

            loop.run_forever()

        except KeyboardInterrupt:
            print("Stopping loop")
            loop.stop()
        finally:
            print("Closing loop")
            loop.close()


if __name__ == "__main__":
    filename = "./test_config.txt"

    controller = TempController(filename)

    controller.run()
