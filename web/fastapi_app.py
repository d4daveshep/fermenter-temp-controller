from typing import Union

from fastapi import FastAPI, Request
from fastapi.responses import HTMLResponse
from fastapi.templating import Jinja2Templates

from controller.config import ControllerConfig
from controller.temperature_database import TemperatureDatabase

app = FastAPI()

config = ControllerConfig("config-test.ini")
temperature_database = TemperatureDatabase(config)

templates = Jinja2Templates(directory="templates")


@app.get("/", response_class=HTMLResponse)
async def read_root(request: Request):
    results_dict = temperature_database.get_last_record()

    results_dict["request"] = request

    # adjust key name as hyphen doesn't work in template
    results_dict["reason"] = results_dict["reason-code"]
    results_dict["brew"] = results_dict["brew-id"]

    return templates.TemplateResponse("root.html", results_dict)

@app.get("/debug")
async def read_root():
    results_dict = temperature_database.get_last_record()
    return results_dict


@app.get("/items/{item_id}")
def read_item(item_id: int, q: Union[str, None] = None):
    return {"item_id": item_id, "q": q}
