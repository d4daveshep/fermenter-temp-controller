from datetime import datetime
import random
from time import sleep

from influxdb_client import InfluxDBClient, Point, WritePrecision
from influxdb_client.client.write_api import SYNCHRONOUS


def random_temp(current_temp):
    direction = random.choice([-1, 0.0, 1])
    movement = random.random()
    return current_temp + direction * movement


# You can generate an API token from the "API Tokens Tab" in the UI
token = "my-super-secret-auth-token"
# org = "my-org"
# bucket = "my-bucket"
org = "daveshep.net"
bucket = "temp-test"

current_fermenter_temp = 20.0
current_ambient_temp = 20.0

# sleep(10)

with InfluxDBClient(url="http://localhost:8086", token=token, org=org) as client:
    write_api = client.write_api(write_options=SYNCHRONOUS)

    # for x in range(60):
    while (True):
        point = Point("temperature") \
            .tag("brew-id", "00-test-v00") \
            .field("fermenter", current_fermenter_temp) \
            .field("ambient", current_ambient_temp) \
            .time(datetime.utcnow(), WritePrecision.MS)

        print(point)

        write_api.write(bucket, org, point)

        current_fermenter_temp = random_temp(current_fermenter_temp)
        current_ambient_temp = random_temp(current_ambient_temp)
        sleep(1)

    # client.close()
