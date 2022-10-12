FROM python:alpine

ADD write_data_to_influxdb.py .
ADD requirements.txt .

RUN pip3 install --no-cache-dir --upgrade -r requirements.txt

CMD ["python3", "./write_data_to_influxdb.py"]
