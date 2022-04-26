FROM python:3.8
WORKDIR /app

COPY requirements.txt .

RUN pip install -r requirements.txt -i https://mirror.sjtu.edu.cn/pypi/web/simple

COPY *.py .
COPY hello.py main.py
