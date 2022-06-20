import os
import sys
import time

import zerorpc
import util

start = time.time()
counter = 0


def report(name, start, end):
    passed_us = (end - start) * 1000000
    print("[%s] duration: %.2f ms" % (str(name), passed_us / 1000))
    sys.stdout.flush()


class RPCServer(object):

    def private_data(self, event):
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

    def tick_rule_start(self):
        global start, counter
        counter = 0
        start = time.time()

    def report_finish_event(self):
        global start, counter
        counter += 1
        end = time.time()
        report("tick %d" % counter, start, end)

    def exit(self):
        global server
        os._exit(0)


s = zerorpc.Server(RPCServer())
s.bind("tcp://0.0.0.0:8090")
s.run()
