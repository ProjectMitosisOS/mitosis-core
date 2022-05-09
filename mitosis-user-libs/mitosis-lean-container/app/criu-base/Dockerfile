FROM ubuntu:16.04

RUN sed -i s/archive.ubuntu.com/mirror.sjtu.edu.cn/g /etc/apt/sources.list && \
    sed -i s/security.ubuntu.com/mirror.sjtu.edu.cn/g /etc/apt/sources.list

RUN chmod 777 /tmp

RUN apt update && apt install -y libcap-dev libnl-3-dev libnet1-dev pkg-config libbsd-dev flex bison libprotobuf-dev libprotobuf-c-dev protobuf-c-compiler protobuf-compiler python-protobuf build-essential git libmount-dev

RUN rm -rf /var/lib/apt/lists/*

RUN GIT_SSL_NO_VERIFY=1 git clone --branch no_error_for_restore https://criu:oDqEoN-nC65FpiYKyC4w@ipads.se.sjtu.edu.cn:1312/distributed-rdma-serverless/mitosis-project/criu.git

RUN cd criu && rm images/google/protobuf/descriptor.proto && ln -s /usr/include/google/protobuf/descriptor.proto images/google/protobuf/descriptor.proto && make -j $(nproc)

RUN apt update && apt install -y python3 wget && wget https://bootstrap.pypa.io/pip/3.5/get-pip.py && python3 get-pip.py && rm -rf /var/lib/apt/lists/*
