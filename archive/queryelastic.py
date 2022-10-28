# learn how to query elasticsearch

import elasticsearch
import requests
from elasticsearch import Elasticsearch

# use a local Elasticsearch instance
ES_HOST = "192.168.1.55"


def search():
    try:
        es = Elasticsearch(
            hosts=[{'host': ES_HOST, 'port': 9200}],
            #       use_ssl=True,
            #       verify_certs=True,
            connection_class=elasticsearch.connection.RequestsHttpConnection)

        # here's a date/time range search filtering out just the timestamp, avg, action and ambient data
        res = es.search(index="test-temp",
                        filter_path=['hits.hits._source.timestamp', 'hits.hits._source.avg', 'hits.hits._source.action',
                                     'hits.hits._source.ambient'],
                        doc_type="temp-reading",
                        body={
                            'from': 0, 'size': 1000,
                            'query': {
                                "range": {
                                    "timestamp": {
                                        "gte": "2018-06-10T16:00:00+12",
                                        "lt": "2018-06-10T17:00:00+12"
                                    }
                                }
                            }
                        })

        # first just get the data records
        records = res['hits']['hits']
        print(len(records))

        # create a data dictionary to store the data in.  key = timestamp, value = dictionary of data
        # e.g. mydata['2018-06-10T16:58:14+1200' = {'avg': 19.92, 'ambient': 15.5, 'action': 'Rest'}
        mydata = {}

        for r in records:
            src = r['_source']
            timestamp = src['timestamp']
            src.pop('timestamp')
            # avg = src['avg']
            # ambient = src['ambient']
            # action = src['action']
            # mydata[timestamp] = {action,avg,ambient}
            mydata[timestamp] = src

        for k in sorted(mydata.keys()):
            print(k, mydata[k])

    except elasticsearch.exceptions.ConnectionError as err:
        # logging.critical("*** ConnectionError *** %s", err)
        raise (err)


if __name__ == '__main__':
    search()
