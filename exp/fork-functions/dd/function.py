import json
import os
import subprocess
import sys
import numpy as np

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'

tmp = '/tmp/'
bs = 'bs=4M'
count = 'count=100'


@tick_execution_time
def handler():
    global start, end

    out_fd = open(tmp + 'io_write_logs', 'w')
    dd = subprocess.Popen(['dd', 'if=/dev/zero', 'of=/dev/zero', bs, count], stderr=out_fd)
    dd.communicate()

    subprocess.check_output(['ls', '-alh', tmp])


@mitosis_bench
def bench():
    handler()


if __name__ == '__main__':
    bench()
