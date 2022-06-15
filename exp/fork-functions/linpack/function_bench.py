from time import time
import random
import string
import pyaes
import json
import os
import sys
from numpy import matrix, linalg, random

sys.path.append("../../common")  # include outer path
from func_bench_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'

def handler():
    n = 4  # FIXME: stuck when n is large
    # LINPACK benchmarks
    ops = (2.0 * n) * n * n / 3.0 + (2.0 * n) * n

    # Create AxA array of random numbers -0.5 to 0.5
    A = random.random_sample((n, n)) - 0.5
    B = A.sum(axis=1)

    # Convert to matrices
    A = matrix(A)
    B = matrix(B.reshape((n, 1)))

    # Ax = B
    x = linalg.solve(A, B)

@func_exec_bench
def bench():
    handler()


if __name__ == '__main__':
    bench()
