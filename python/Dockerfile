FROM python:alpine

COPY *.py ./
COPY test*.txt ./
COPY requirements.txt .

RUN pip3 install --no-cache-dir --upgrade -r requirements.txt

CMD ["python3", "-m", "pytest", "-v", "test_config_file.py", "test_database.py", "test_temp_controller.py"]
