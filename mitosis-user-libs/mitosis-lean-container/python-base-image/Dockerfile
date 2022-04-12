FROM python:3.8

RUN pip install igraph Pillow torch numpy torchvision chameleon six pyaes -i https://mirror.sjtu.edu.cn/pypi/web/simple

RUN sed -i s/deb.debian.org/mirrors.aliyun.com/g /etc/apt/sources.list && \
    sed -i s/security.debian.org/mirrors.aliyun.com/g /etc/apt/sources.list && \
    apt update && \
    apt install -y ffmpeg && \
    rm -rf /var/lib/apt/lists/*
