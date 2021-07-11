import argparse
import logging
from datetime import timedelta

import numpy as np
import pandas as pd
import rfc3339 as rfc3339
from influxdb import DataFrameClient
from scipy import stats


def analyse_db(db_name, timeframe="12h", host="localhost", port=8086):
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

    logging.info(f"Analysing last {timeframe}")
    query = "select * from temperature where time >= now() - " + timeframe
    logging.debug("Running query: " + query)

    # run the query and load the result set into a dataframe
    rs = client.query(query)
    df = pd.DataFrame(rs['temperature'])

    # convert time index to NZ timezone
    df.index = df.index.tz_convert('Pacific/Auckland')

    logging.info("===========================")
    logging.info("Ambient temperature data")
    logging.info("---------------------------")
    logging.debug(f"Got {df['ambient_temp'].count():d} ambient temp records...")

    logging.info(f"min ambient = {df['ambient_temp'].min():.2f} at {df['ambient_temp'].idxmin():%Y-%m-%d %H:%M}")
    logging.info(f"max ambient = {df['ambient_temp'].max():.2f} at {df['ambient_temp'].idxmax():%Y-%m-%d %H:%M}")
    logging.info(f"average ambient = {df['ambient_temp'].mean():.2f}")
    logging.info(f"std dev ambient = {df['ambient_temp'].std():.2f}")

    logging.info("===========================")
    logging.info("Fermenter temperature data")
    logging.info("---------------------------")
    logging.debug(f"Got {df['fermenter_temp'].count():d} fermenter temp records...")

    logging.info(f"min fermenter = {df['fermenter_temp'].min():.2f} at {df['fermenter_temp'].idxmin():%Y-%m-%d %H:%M}")
    logging.info(f"max fermenter = {df['fermenter_temp'].max():.2f} at {df['fermenter_temp'].idxmax():%Y-%m-%d %H:%M}")
    logging.info(f"average fermenter = {df['fermenter_temp'].mean():.2f}")
    logging.info(f"std dev fermenter = {df['fermenter_temp'].std():.2f}")

    # calculate zscore to identify outliers
    temps = df['fermenter_temp']  # this is a Series
    zscores = stats.zscore(temps)
    abs_zscores = np.abs(zscores)

    outliers = (abs_zscores < 3).groupby(level=0).all()  # this gives us a Series with True/False values
    # logging.debug(outliers)

    new_df = df[outliers]  # don't really understand how this works but it removes the False values (i.e. outliers)
    logging.debug(f"After removing outliers we now have {new_df['fermenter_temp'].count():d} records")

    logging.info("===========================")
    logging.info("Updated fermenter temperature data")
    logging.info("---------------------------")
    logging.debug(f"Got {new_df['fermenter_temp'].count():d} fermenter temp records...")

    logging.info(
        f"min fermenter = {new_df['fermenter_temp'].min():.2f} at {new_df['fermenter_temp'].idxmin():%Y-%m-%d %H:%M}")
    logging.info(
        f"max fermenter = {new_df['fermenter_temp'].max():.2f} at {new_df['fermenter_temp'].idxmax():%Y-%m-%d %H:%M}")
    logging.info(f"average fermenter = {new_df['fermenter_temp'].mean():.2f}")
    logging.info(f"std dev fermenter = {new_df['fermenter_temp'].std():.2f}")

    logging.info("===========================")

    # Calculate the heat start lag
    # find heat start times
    logging.info("Finding heat start lag")
    logging.info("======================")
    query = "select fermenter_temp, change_action from temperature where change_action='START HEATING' and time >= now() - " + timeframe

    logging.debug("Running query: " + query)
    rs = client.query(query)
    df = pd.DataFrame(rs['temperature'])
    df.index = df.index.tz_convert('Pacific/Auckland')
    logging.debug(df)

    for index, row in df.iterrows():
        heat_stop_temp = row['fermenter_temp']
        logging.debug(f"Heat stop temp = {heat_stop_temp:.2f}")

        time0 = rfc3339.rfc3339(index)
        time5 = rfc3339.rfc3339(index + timedelta(minutes=5))

        # logging.debug(f"Heat start = {time0}")
        # logging.debug(f"End window = {time5}")
        query = "select min(fermenter_temp) from temperature where '" + time0 + "' <= time and time <= '" + time5 + "'"
        # logging.debug(query)
        rs1 = client.query(query)
        df1 = pd.DataFrame(rs1['temperature'])
        df1.index = df1.index.tz_convert('Pacific/Auckland')
        min_temp_after_heat_stop = df1.iloc[0]['min']
        logging.debug(f"Min temp after heat stop = {min_temp_after_heat_stop:.2f}")





    # logging.debug(df.index)

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
    parser.add_argument('-tf', '--timeframe', type=str, required=False, default='12h', help='timeframe to analyse')
    parser.add_argument('--host', type=str, required=False, default='localhost', help='hostname of InfluxDB http API')
    parser.add_argument('--port', type=int, required=False, default=8086, help='port of InfluxDB http API')

    return parser.parse_args()


if __name__ == '__main__':
    # Set up logging to console
    logging.basicConfig(level=logging.DEBUG,
                        format="%(levelname)s: %(asctime)s: %(message)s")
    logging.debug("here I am")

    args = parse_args()  # database name is a mandatory parameter

    # run the analysis function
    analyse_db(db_name=args.db_name, timeframe=args.timeframe, host=args.host, port=args.port)
