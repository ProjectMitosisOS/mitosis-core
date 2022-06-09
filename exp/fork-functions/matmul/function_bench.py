import json
import os
import sys
import numpy as np

sys.path.append("../../common")  # include outer path
from func_bench_wrapper import *

def handler():
    n = 64
    A = np.random.rand(n, n)
    B = np.random.rand(n, n)
    C = np.matmul(A, B)

@func_exec_bench
def bench():
    handler()


if __name__ == '__main__':
    bench()
