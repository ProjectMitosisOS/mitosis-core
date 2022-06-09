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
parser.add_argument("-thread_num", type=int, default=1, help="thread number")
args = parser.parse_args()

bench_run_sec = args.run_sec
thread_num = args.thread_num
op_cnt_list = [0 for _ in range(thread_num)]
running = True


class StatisticThread(threading.Thread):
    def __init__(self, run_sec):
        super(StatisticThread, self).__init__()
        self.run_sec = run_sec
        self.prev_op = 0
        self.tick = time.time()

    def run(self):
        global running, op_cnt_list
        for i in range(self.run_sec):
            time.sleep(1)
            cur_op = sum(op_cnt_list)
            ops = cur_op - self.prev_op
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
            latency_ms = (passed_us / 1e3) / (float(op) / thread_num)
            qps = (float(op) / passed_us) * 1e6
            print("Throughput: %ld containers/sec, latency %f ms" % (qps, latency_ms))


class BenchThread(threading.Thread):
    def __init__(self, tid, run_sec, handler):
        super(BenchThread, self).__init__()
        self.tid = tid
        self.run_sec = run_sec
        self.handler = handler

    def run(self):
        global running
        while running:
            global op_cnt_list
            self.handler()
            op_cnt_list[self.tid] += 1


def func_exec_bench(handler):
    @wraps(handler)
    def wrapper(*args, **kwargs):
        statisticT = StatisticThread(run_sec=bench_run_sec)
        statisticT.start()
        threads = [BenchThread(i, bench_run_sec, handler) for i in range(thread_num)]
        for t in threads:
            t.start()

    return wrapper
