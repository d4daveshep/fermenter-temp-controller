from controller.temperature import TemperatureReading, ControllerAction
import json
import pytest


@pytest.fixture
def json_string() -> str:
    return '{\
"instant":21.0,\
"average":21.3,\
"min":18.9,\
"max":23.4,\
"target":20.0,\
"ambient":12.3,\
"action":"Cool",\
"rest":null,\
"heat":null,\
"cool":true,\
"reason-code":"RC5.1",\
"json-size":123\
}'


def test_temperature_reading(json_string: str):
    temp_reading: TemperatureReading = TemperatureReading.model_validate_json(
        json_string
    )

    assert temp_reading.instant == 21.0
    assert temp_reading.average == 21.3
    assert temp_reading.min == 18.9
    assert temp_reading.max == 23.4
    assert temp_reading.target == 20.0
    assert temp_reading.ambient == 12.3
    assert temp_reading.action == ControllerAction.COOL
    assert temp_reading.cool is True
    assert temp_reading.reason_code == "RC5.1"
    assert temp_reading.json_size == 123


def test_serialisation_of_temperature_reading(json_string: str):
    temp_reading: TemperatureReading = TemperatureReading.model_validate_json(
        json_string
    )

    serialised_string: str = temp_reading.model_dump_json(by_alias=True)
    assert serialised_string == json_string

    temp_reading_2: TemperatureReading = TemperatureReading.model_validate_json(
        serialised_string
    )
    assert temp_reading == temp_reading_2
