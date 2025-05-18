from enum import StrEnum
from typing import Optional

from pydantic import BaseModel, Field


class ControllerAction(StrEnum):
    NO_ACTION = "No Action"
    REST = "Rest"
    HEAT = "Heat"
    COOL = "Cool"
    ACTION_ERROR = "Error"


class TemperatureReading(BaseModel):
    instant: float
    average: float
    min: float
    max: float
    target: float
    ambient: float
    action: ControllerAction
    rest: Optional[bool] = None
    heat: Optional[bool] = None
    cool: Optional[bool] = None
    reason_code: str = Field(alias="reason-code")
    json_size: int = Field(alias="json-size")
