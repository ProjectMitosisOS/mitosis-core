import json
import os
import sys
import numpy as np

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'


@tick_execution_time
def handler():
    n = 64
    A = np.random.rand(n, n)
    B = np.random.rand(n, n)
    C = np.matmul(A, B)


@mitosis_bench
def bench():
    handler()


if __name__ == '__main__':
    bench()
