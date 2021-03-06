import argparse

import numpy as np
import pandas as pd
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
    print("influxdb dataframe client created")

    dbname = "99-TEST-v99"
    db_list = client.get_list_database()

    check_name = {"name": dbname}
    if check_name in db_list:
        client.switch_database(dbname)
        print("using database " + dbname)

    else:
        print("can't find database: " + dbname)
        exit(-1)

    query = "select * from temperature"  # where change_action='START HEATING'"
    print("running query: " + query)

    rs = client.query(query)
    # print("resultset is...")
    # print(rs)
    # print("keys are... ", rs.keys())
    # print("temperature values...")
    # print(rs['temperature'])

    # load the result set into a dataframe
    df = pd.DataFrame(rs['temperature'])
    # convert time index to NZ timezone
    df.index = df.index.tz_convert('Pacific/Auckland')

    # print("dataframe is...")
    # print(df)

    print("from ", df['ambient_temp'].count(), " records...")
    print("min ambient = ", df['ambient_temp'].min())
    print("average ambient = ", df['ambient_temp'].mean())

    # print(df['fermemter_temp'].count(), "records")
    # temps = df['ambient_temp']  # this is a Series
    # print("ambient temp std dev =", temps.std())
    # for i in temps:
    #     if not np.isnan(i):
    #         print(i)

    # copy the fermeMter values to the fermeNter column
    print("fixing the fermenter temp data")
    for index, row in df.iterrows():
        if not np.isnan(row['fermemter_temp']):
            df.at[index, 'fermenter_temp'] = row['fermemter_temp']

    print("from", df['fermenter_temp'].count(), "records")
    print("min fermenter temp = ", df['fermenter_temp'].min(), "at", df['fermenter_temp'].idxmin())
    print("removing this value")
    df = df.drop(index=df['fermenter_temp'].idxmin())
    print("min fermenter temp = ", df['fermenter_temp'].min(), "at", df['fermenter_temp'].idxmin())
    print("average fermenter temp = ", df['fermenter_temp'].mean())
    print("max fermenter temp = ", df['fermenter_temp'].max(), "at", df['fermenter_temp'].idxmax())
    temps = df['fermenter_temp']  # this is a Series
    print("fermenter temp std dev =", temps.std())


if __name__ == '__main__':
    print("in main")
    args = parse_args()
    # main(host=args.host, port=args.port)
    do_dataframes(host=args.host, port=args.port)
