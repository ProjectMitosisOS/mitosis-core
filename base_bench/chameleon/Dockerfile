FROM python:3.8

COPY requirements.txt .

RUN pip install --progress-bar off  -r requirements.txt -i https://mirror.sjtu.edu.cn/pypi/web/simple

COPY *.py .
COPY chameleon.py main.py

CMD ["python", "main.py"]
