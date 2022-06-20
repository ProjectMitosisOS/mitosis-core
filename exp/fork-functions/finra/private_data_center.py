import sys
import time

import zerorpc
import util
from agileutil.rpc.server import rpc, TcpRpcServer


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


server = TcpRpcServer('0.0.0.0', 8090)
server.serve()