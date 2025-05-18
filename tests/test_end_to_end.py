from controller.config import ControllerConfig
from pathlib import Path


def test_end_to_end_sync():
    # read the configuration
    config: ControllerConfig = ControllerConfig.load_config(
        filename=Path("tests/valid_config.ini")
    )
    # get temperature readings from sensor
    # put temperature readings in database
    # read latest readings from database
    # update the target temperature
    # get temperature readings from sensor
    # put temperature readings in database
    # read latest readings from database
    # update the brew_id
    # get temperature readings from sensor
    # put temperature readings in database
    # read latest readings from database
    assert False
