import os
import sys
import time

sys.path.append("../../common")  # include outer path

import syscall_lib
import bench
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'


@tick_execution_time
def handler(working_sz):
    pass


@mitosis_bench
def bench():
    handler(working_sz=working_set)


if __name__ == '__main__':
    bench()
