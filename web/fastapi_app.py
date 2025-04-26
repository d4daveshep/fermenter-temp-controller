import json
from datetime import datetime
from pathlib import Path
from pytz import timezone
from fastapi import FastAPI, Request, Form, status
from fastapi.responses import HTMLResponse, RedirectResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates

from controller.config import ControllerConfig, EnvSettings
from controller.temperature_database import TemperatureDatabase
from controller.zmq_sender import ZmqSender

app = FastAPI()

app.mount(
    "/static",
    StaticFiles(directory=Path(__file__).parent.parent.absolute() / "static"),
    name="static",
)

settings = EnvSettings()
config = ControllerConfig(settings.config_filename)
temperature_database = TemperatureDatabase(config)
sender = ZmqSender(config)

nztz = timezone("Pacific/Auckland")

templates = Jinja2Templates(directory="templates")


@app.get("/", response_class=HTMLResponse)
async def root(request: Request):
    now = datetime.now(nztz)
    results_dict = temperature_database.get_last_record()
    results_dict["timestamp"] = now.strftime("%d %b %Y %H:%M:%S")
    results_dict["request"] = request

    # adjust key name as hyphen doesn't work in template
    if "reason-code" in results_dict.keys():
        results_dict["reason"] = results_dict["reason-code"]
    if "brew-id" in results_dict.keys():
        results_dict["brew"] = results_dict["brew-id"]

    return templates.TemplateResponse("root.html", results_dict)


@app.get("/debug")
async def debug():
    results_dict = temperature_database.get_last_record()
    return results_dict


@app.get("/update-target-temp/{temp}")
async def update_target_temp(temp: float):
    new_target_temp = temp
    config.logger.info(f"setting target temp to {new_target_temp}")

    zmq_message_to_send = json.dumps({"new-target-temp": new_target_temp})
    try:
        await sender.send_string(zmq_message_to_send)
    except Exception as err_info:
        config.logger.error(err_info)
        raise

    return {"new-target": new_target_temp}


@app.get("/new-target-temp", response_class=HTMLResponse)
async def display_target_temp_form(request: Request):
    results_dict = temperature_database.get_last_record()
    request_dict = {"request": request, "target": results_dict["target"]}

    return templates.TemplateResponse("new_target_temp_form.html", request_dict)


@app.post("/post-target-temp/")
async def send_new_target_temp(request: Request, temp: float = Form()):
    new_target_temp = temp
    config.logger.info(f"setting target temp to {new_target_temp}")

    zmq_message_to_send = json.dumps({"new-target-temp": new_target_temp})
    try:
        await sender.send_string(zmq_message_to_send)
    except Exception as err_info:
        config.logger.error(err_info)
        raise

    return RedirectResponse(request.url_for("root"), status_code=status.HTTP_303_SEE_OTHER)


@app.get("/new-brew-id", response_class=HTMLResponse)
async def display_brew_id_form(request: Request):
    results_dict = temperature_database.get_last_record()
    request_dict = {"request": request, "brew": results_dict["brew-id"]}

    return templates.TemplateResponse("new_brew_id_form.html", request_dict)


@app.post("/post-brew-id/")
async def send_new_brew_id(request: Request, brew_id: str = Form()):
    new_brew_id = brew_id
    config.logger.info(f"setting brew id to {new_brew_id}")

    zmq_message_to_send = json.dumps({"new-brew-id": new_brew_id})
    try:
        await sender.send_string(zmq_message_to_send)
    except Exception as err_info:
        config.logger.error(err_info)
        raise

    return RedirectResponse(request.url_for("root"), status_code=status.HTTP_303_SEE_OTHER)
