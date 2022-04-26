import os
import time

import syscall_lib

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'
import numpy as np

dump_key = 73


def handler():
    global start, end
    start = time.time()
    n = 100
    A = np.random.rand(n, n)
    B = np.random.rand(n, n)
    C = np.matmul(A, B)
    end = time.time()


def checkpoint(key):
    fd = syscall_lib.open()
    syscall_lib.call_prepare(fd, key)


if __name__ == '__main__':
    handler()
    checkpoint(dump_key)
    handler()
