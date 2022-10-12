import pytest
import requests as requests


def test_influxdb_server_is_available():
    # run equivalant of curl -sL -I localhost:8086/ping?wait_for_leader=30s

    URL = "http://localhost:8086/ping"
    PARAMS = {'wait_for_leader': "30s"}

    try:
        response = requests.get(url = URL, params = PARAMS)
        assert response.status_code == 204
        assert response.headers["X-Influxdb-Build"] == "OSS"
        assert response.headers["X-Influxdb-Version"] >= "v2.4"
    except requests.ConnectionError as err:
        assert False, err
    except Exception as exec:
        assert False, exec
