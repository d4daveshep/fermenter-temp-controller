import argparse
import pandas

from influxdb import InfluxDBClient, DataFrameClient


def main(host="localhost", port=8086):
    client = InfluxDBClient(host, port)
    print("database client created")

    dbname = "test_temp"
    db_list = client.get_list_database()

    check_name = {"name": dbname}
    if check_name in db_list:
        client.switch_database(dbname)
        print("using database " + dbname)
        policies = client.get_list_retention_policies()
        print("retention policies are ", policies)

    else:
        client.create_database(dbname)
        print("created database " + dbname)
        client.create_retention_policy("my_policy", "3d", 1, dbname, default=True)
        print("set 4 week retention policy")

    json_body = [
        {
            "measurement": "temperature",
            "tags": {
                "brew_id": "99-TEST-v99"
            },
            "time": "2021-07-02T17:44:40.471836+12:00",
            "fields": {
                "ambient_temp": 12.31, "fermenter_temp": 20.12, "target_temp": 20.0, "controller_action": "REST"
            }
        }
    ]
    client.write_points(json_body, database=dbname)

    client.close()
    print("database client closed")


def parse_args():
    """Parse the args."""
    parser = argparse.ArgumentParser(
        description='example code to play with InfluxDB')
    parser.add_argument('--host', type=str, required=False,
                        default='localhost',
                        help='hostname of InfluxDB http API')
    parser.add_argument('--port', type=int, required=False, default=8086,
                        help='port of InfluxDB http API')
    return parser.parse_args()


def do_dataframes(host="localhost", port=8086):
    client = DataFrameClient(host, port)
    print("dataframe client created")

    dbname = "99-TEST-v99-1234"
    db_list = client.get_list_database()

    check_name = {"name": dbname}
    if check_name in db_list:
        client.switch_database(dbname)
        print("using database " + dbname)

    else:
        print("can't find database: " + dbname)
        exit(-1)

if __name__ == '__main__':
    args = parse_args()
    # main(host=args.host, port=args.port)
    do_dataframes(host=args.host, port=args.port)
