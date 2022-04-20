import os
import sys
import time

sys.path.append("../../common")  # include outer path
import syscall_lib
from bench import report

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


def checkpoint():
    fd = syscall_lib.open()
    syscall_lib.call_prepare(fd, dump_key)


if __name__ == '__main__':
    global start, end
    handler()
    report("execution", start, end)

    checkpoint()
    handler()
    report("execution", start, end)
    while True:
        time.sleep(1)
