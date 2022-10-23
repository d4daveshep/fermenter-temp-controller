from os.path import exists

from temp_controller import TempController


def test_create_temperature_controller_object():
    filename = "./test_valid_config_file.txt"
    assert exists(filename)

    controller = TempController(filename)
    controller_config = controller.config
    assert controller_config

    temperature_database = controller.temperature_database
    assert temperature_database.is_server_available()

    assert False # TODO finish this
