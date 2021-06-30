import argparse

from influxdb import InfluxDBClient


def main(host="localhost", port=8086):
    client = InfluxDBClient(host, port)
    print("database client created")

    dbname = "test_temp"
    if dbname in client.get_list_database():
        client.switch_database(dbname)
        print("using database " + dbname)
    else:
        client.create_database(dbname)
        print("created database " + dbname)

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


if __name__ == '__main__':
    args = parse_args()
    main(host=args.host, port=args.port)
