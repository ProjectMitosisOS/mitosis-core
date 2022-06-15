FROM python:3.8

COPY . ./
RUN pip install -r requirements.txt -i https://mirror.sjtu.edu.cn/pypi/web/simple
RUN g++ lean_child.cc -o lean_child
RUN g++ nil_rpc.cc -o nil_rpc

