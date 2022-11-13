import json
import random
from typing import Union

from fastapi import FastAPI, Request
from fastapi.responses import HTMLResponse
from fastapi.templating import Jinja2Templates

from controller.config import ControllerConfig
from controller.temperature_database import TemperatureDatabase
from controller.zmq_sender import ZmqSender

app = FastAPI()

config = ControllerConfig("config-test.ini")
temperature_database = TemperatureDatabase(config)
sender = ZmqSender(config)

templates = Jinja2Templates(directory="templates")


@app.get("/", response_class=HTMLResponse)
async def read_root(request: Request):
    results_dict = temperature_database.get_last_record()
    results_dict["request"] = request

    # adjust key name as hyphen doesn't work in template
    if "reason-code" in results_dict.keys():
        results_dict["reason"] = results_dict["reason-code"]
    if "brew-id" in results_dict.keys():
        results_dict["brew"] = results_dict["brew-id"]

    return templates.TemplateResponse("root.html", results_dict)


@app.get("/debug")
async def read_debug():
    results_dict = temperature_database.get_last_record()
    return results_dict


@app.get("/items/{item_id}")
def read_item(item_id: int, q: Union[str, None] = None):
    return {"item_id": item_id, "q": q}


@app.get("/update-target-temp/{temp}")
async def update_target_temp(temp: float):
    new_target_temp = temp
    # new_target_temp = float(random.randrange(10, 30))
    config.logger.info(f"setting target temp to {new_target_temp}")

    zmq_message_to_send = json.dumps({"new-target-temp": new_target_temp})
    try:
        await sender.send_string(zmq_message_to_send)
    except Exception as err_info:
        config.logger.error(err_info)
        raise

    return {"new-target": new_target_temp}
