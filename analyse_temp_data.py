import argparse
import logging

import numpy as np
import pandas as pd
from influxdb import DataFrameClient


def analyse_db(db_name, host="localhost", port=8086):
    logging.info("Analysing temperature database: " + db_name)
    client = DataFrameClient(host, port)
    logging.debug("InfluxDB dataframe client created")

    # db_name = "99-TEST-v99"
    db_list = client.get_list_database()

    check_name = {"name": db_name}
    if check_name in db_list:
        client.switch_database(db_name)
        logging.debug("using database " + db_name)

    else:
        logging.critical("Can't find database: " + db_name)
        exit(-1)

    # query last 24hrs - this will return 24*60*60/10 = 8640 records
    logging.info("Analysing last 24 hours")
    query = "select * from temperature where time >= now() - 24h"
    logging.debug("Running query: " + query)

    # run the query and load the result set into a dataframe
    rs = client.query(query)
    df = pd.DataFrame(rs['temperature'])

    # convert time index to NZ timezone
    df.index = df.index.tz_convert('Pacific/Auckland')

    logging.info("Ambient temperature data")
    logging.info("------------------------")
    logging.debug(f"Got {df['ambient_temp'].count():d} ambient temp records...")

    logging.info(f"min ambient = {df['ambient_temp'].min():.2f}")
    logging.info(f"min ambient = {df['ambient_temp'].max():.2f}")
    logging.info(f"average ambient = {df['ambient_temp'].mean():.2f}")
    logging.info(f"std dev ambient = {df['ambient_temp'].std():.2f}")

    #
    # print("from", df['fermenter_temp'].count(), "records")
    # print("min fermenter temp = ", df['fermenter_temp'].min(), "at", df['fermenter_temp'].idxmin())
    # print("removing this value")
    # df = df.drop(index=df['fermenter_temp'].idxmin())
    # print("min fermenter temp = ", df['fermenter_temp'].min(), "at", df['fermenter_temp'].idxmin())
    # print("average fermenter temp = ", df['fermenter_temp'].mean())
    # print("max fermenter temp = ", df['fermenter_temp'].max(), "at", df['fermenter_temp'].idxmax())
    # temps = df['fermenter_temp']  # this is a Series
    # print("fermenter temp std dev =", temps.std())


def parse_args():
    # Parse the args
    parser = argparse.ArgumentParser(
        description='Analyse fermenter temperature data')
    parser.add_argument('db_name', type=str, help="name of the InfluxDB database to analyse")  # mandatory
    parser.add_argument('--host', type=str, required=False, default='localhost', help='hostname of InfluxDB http API')
    parser.add_argument('--port', type=int, required=False, default=8086, help='port of InfluxDB http API')

    return parser.parse_args()


if __name__ == '__main__':
    # Set up logging to console
    logging.basicConfig(level=logging.INFO,
                        format="%(levelname)s: %(asctime)s: %(message)s")
    logging.debug("here I am")

    args = parse_args()  # database name is a mandatory parameter

    # run the analysis function
    analyse_db(db_name=args.db_name, host=args.host, port=args.port)
