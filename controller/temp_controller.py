import asyncio
import json
import logging
from asyncio import StreamReader, StreamWriter

import serial_asyncio

from config import ControllerConfig
from temperature_database import TemperatureDatabase


class TempController:
    def __init__(self, config_filename: str):
        self.logger = None
        self.config = ControllerConfig(config_filename)
        self.temperature_database = TemperatureDatabase(self.config)
        self.current_target_temp = 0.0

        self.serial_port_writer = None
        self.serial_port_reader = None

        self.configure_logging()

    @classmethod
    async def open_serial_connection(cls, port_url: str) -> (StreamReader, StreamWriter):
        return await serial_asyncio.open_serial_connection(url=port_url, baudrate=115200)

    async def read_line_from_serial(self) -> str:
        line = await self.serial_port_reader.readline()
        self.logger.debug(f"read {line} from serial")
        return str(line, 'utf-8')

    async def write_float_to_serial_port(self, float_num: float) -> None:
        string_to_write = '<' + str(float_num) + '>'
        self.logger.info(f"writing {string_to_write} to serial")
        self.serial_port_writer.write(string_to_write.encode())
        await asyncio.sleep(0)

    @classmethod
    def convert_dict_int_values_to_float(cls, json_dict: dict) -> dict:

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

                data_dict = self.convert_dict_int_values_to_float(json_dict)

                # store some key data fields
                self.current_target_temp = data_dict["target"]

                point = self.temperature_database.create_point_from_fermenter_json_dict(json_dict)
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
            self.serial_port_reader, self.serial_port_writer = loop.run_until_complete(
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

    def configure_logging(self):
        self.logger = logging.getLogger("stdout")
        self.logger.setLevel(logging.DEBUG)
        console_handler = logging.StreamHandler()
        # console_handler.setLevel(logging.DEBUG)
        formatter = logging.Formatter("%(levelname)s: %(asctime)s: %(message)s")
        console_handler.setFormatter(formatter)
        self.logger.addHandler(console_handler)


if __name__ == "__main__":
    filename = "config-test.ini"

    controller = TempController(filename)

    controller.run()
