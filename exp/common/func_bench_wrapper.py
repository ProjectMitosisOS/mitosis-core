import argparse
import os
import sys
import time
from functools import wraps
import syscall_lib
import bench
import time
import threading

parser = argparse.ArgumentParser()
parser.add_argument("-run_sec", type=int, default=10, help="running seconds")
parser.add_argument("-working_set", type=int, default=16777216,
                    help="working set size")
args = parser.parse_args()

bench_run_sec = args.run_sec
working_set = args.working_set
total_op = 0
running = True


class StatisticThread(threading.Thread):
    def __init__(self, run_sec):
        super(StatisticThread, self).__init__()
        self.run_sec = run_sec
        self.prev_op = 0
        self.tick = time.time()

    def run(self):
        global running, total_op
        for i in range(self.run_sec):
            time.sleep(1)
            cur_op = total_op
            ops = cur_op - self.prev_op
            if ops > 0:
                self.prev_op = cur_op
                end = time.time()
                self.report(self.tick, end, ops)
                self.tick = time.time()
        running = False

    def report(self, start, end, op):
        if op == 0:
            print("Throughput: %ld containers/sec, latency %f ms" % (0, 0))
        else:
            passed_us = (end - start) * 1e6
            latency_ms = (passed_us / 1e3) / (float(op))
            qps = (float(op) / passed_us) * 1e6
            print("Throughput: %ld containers/sec, latency %f ms" % (qps, latency_ms))


def func_exec_bench(handler):
    @wraps(handler)
    def wrapper(*args, **kwargs):
        statisticT = StatisticThread(run_sec=bench_run_sec)
        statisticT.start()

        while running:
            global total_op
            handler(*args, **kwargs)
            total_op += 1
    return wrapper
