FROM python:3.8
WORKDIR /app

COPY lean_child.cc main.cc
COPY *.h ./

RUN g++ main.cc -o lean_child

