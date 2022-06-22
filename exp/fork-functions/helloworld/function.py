import os
import sys
import time
import mmap

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'


@tick_execution_time
def handler():
    if profile == 1:
        print("hello world")
    else:
        s = "hello world"
        t = s.find('d')


@mitosis_bench
def bench():
    handler()


if __name__ == '__main__':
    bench()
