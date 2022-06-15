import os
import sys
import time
import mmap

sys.path.append("../../common")  # include outer path
from func_bench_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'


def handler():
    s = "hello world"
    s.find("d")

@func_exec_bench
def bench():
    handler()


if __name__ == '__main__':
    bench()
