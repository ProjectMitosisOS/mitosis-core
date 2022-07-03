FROM python:2.7

COPY requirements.txt .

RUN pip install -r requirements.txt -i https://mirror.sjtu.edu.cn/pypi/web/simple

COPY function.py main.py
