import os
import random
import sys
import time

import numpy as np

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'
from numpy import matrix, linalg, random

sys.path.append("../../common")  # include outer path

import syscall_lib
import bench

## Migration related
app_name = "linpack"

import argparse

parser = argparse.ArgumentParser()
parser.add_argument("-handler_id", type=int, default=101, help="rfork handler id")
parser.add_argument("-profile", type=int, default=1, help="whether print out the profile data")
parser.add_argument("-pin", type=int, default=0, help="whether pin the descriptor in kernel")
args = parser.parse_args()

handler_id = args.handler_id
profile = args.profile
pin = args.pin


## Migration related end

def handler():
    global start, end
    start = time.time()

    ## Body start
    n = 1000
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
    ## Body end

    end = time.time()
    if profile == 1:
        bench.report("%s-execution" % app_name, start, end)


def checkpoint(key):
    global start, end
    fd = syscall_lib.open()
    start = time.time()
    if pin == 1:
        syscall_lib.call_prepare_ping(fd, key)
    else:
        syscall_lib.call_prepare(fd, key)
    end = time.time()
    if profile == 1:
        bench.report("%s-prepare" % app_name, start, end)


if __name__ == '__main__':
    handler()
    checkpoint(handler_id)
    handler()
    os._exit(0)
