import gzip
import os
import shutil
import sys

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'


@tick_execution_time
def lambda_handler():
    dst = 'result'
    src = 'compression'
    shutil.make_archive(dst, 'zip', src)


@mitosis_bench
def bench():
    lambda_handler()


if __name__ == '__main__':
    bench()
