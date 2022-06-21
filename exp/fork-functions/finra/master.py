import os
import sys
import time

import numpy as np
import zerorpc
import util

def report(name, start, end):
    passed_us = (end - start) * 1000000
    print("[%s] duration: %.2f ms" % (str(name), passed_us / 1000))
    sys.stdout.flush()


class RPCServer(object):
    def __init__(self):
        self.process = 0
        self.start = time.time()
        self.finish_cnt = 0

    def tick_rule_start(self, process):
        self.finish_cnt = 0
        self.process = process
        self.start = time.time()

    def report_finish_event(self):
        self.finish_cnt += 1
        if self.finish_cnt == self.process:
            end = time.time()
            report("rule %d" % self.process, self.start, end)


s = zerorpc.Server(RPCServer())
s.bind("tcp://0.0.0.0:8090")
s.run()
