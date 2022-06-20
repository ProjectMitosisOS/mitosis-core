import argparse
import socket
import sys
import time

import zerorpc

parser = argparse.ArgumentParser()
parser.add_argument("-port", type=int, default=8080, help="rpc server port")
parser.add_argument("-remote_host", type=str, default="localhost", help="rpc server host")
parser.add_argument("-process", type=int, default=1, help="rpc parallel num")
args = parser.parse_args()
port = args.port
process = args.process
remote_host = args.remote_host
from agileutil.rpc.client import TcpRpcClient

clients = []
master_cli = TcpRpcClient(servers=["%s:%d" % (remote_host, 8090)])

s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

# start
master_cli.call("tick_rule_start")

# Trigger without waiting
for i in range(process):
    s.sendto(b"data", (remote_host, port + i))
