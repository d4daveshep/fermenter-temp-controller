from config import ControllerConfig


def loop(config: ControllerConfig):
    target_temp = config.target_temp
    while (True):

        if config.target_temp != target_temp:
            # write new target_temp to arduino
            pass







