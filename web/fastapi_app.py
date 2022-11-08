from typing import Union

from fastapi import FastAPI

from controller.config import ControllerConfig
from controller.temperature_database import TemperatureDatabase

app = FastAPI()

config = ControllerConfig("config-test.ini")
temperature_database = TemperatureDatabase(config)

@app.get("/")
def read_root():
    results_dict = temperature_database.get_last_record()
    return results_dict
    # return {"Hello": "World"}


@app.get("/items/{item_id}")
def read_item(item_id: int, q: Union[str, None] = None):
    return {"item_id": item_id, "q": q}
