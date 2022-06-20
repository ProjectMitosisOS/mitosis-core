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
