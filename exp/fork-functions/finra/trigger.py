import argparse
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


def report(name, start, end):
    passed_us = (end - start) * 1000000
    print("[%s] duration: %.2f ms" % (str(name), passed_us / 1000))
    sys.stdout.flush()


clients = []

cli = TcpRpcClient(servers=["%s:%d" % (remote_host, port + i) for i in range(process)], timeout=10)
start = time.time()
res = cli.call("handler")  # TODO: async rpc
end = time.time()
report("trigger", start, end)
