import os
import sys
import time

import numpy as np
import zerorpc
import util

start = time.time()
finish_list = []


def report(name, start, end):
    passed_us = (end - start) * 1000000
    print("[%s] duration: %.2f ms" % (str(name), passed_us / 1000))
    sys.stdout.flush()


class RPCServer(object):
    def tick_rule_start(self, process):
        global start, finish_list
        finish_list = [False for _ in range(process)]
        start = time.time()

    def report_finish_event(self, key):
        global finish_list
        finish_list[key] = True
        if np.all(finish_list):
            end = time.time()
            report("rule %d" % len(finish_list), start, end)


s = zerorpc.Server(RPCServer())
s.bind("tcp://0.0.0.0:8090")
s.run()
