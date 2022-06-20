import sys
import time

import zerorpc
import util
from agileutil.rpc.server import rpc, TcpRpcServer

start = time.time()
counter = 0


def report(name, start, end):
    passed_us = (end - start) * 1000000
    print("[%s] duration: %.2f ms" % (str(name), passed_us / 1000))
    sys.stdout.flush()


@rpc
def private_data(event):
    startTime = 1000 * time.time()

    portfolio = event['body']['portfolio']

    data = util.portfolios[portfolio]

    valid = True

    for trade in data:
        side = trade['Side']
        # Tag ID: 552, Tag Name: Side, Valid values: 1,2,8
        if not (side == 1 or side == 2 or side == 8):
            valid = False
            break

    response = {'statusCode': 200, 'body': {'valid': valid, 'portfolio': portfolio}}
    endTime = 1000 * time.time()
    return util.timestamp(response, event, startTime, endTime, 0)


@rpc
def tick_rule_start():
    global start
    start = time.time()


@rpc
def report_finish_event():
    global start, counter
    counter += 1
    end = time.time()
    report("tick %d" % counter, start, end)


server = TcpRpcServer('0.0.0.0', 8090)
server.serve()
