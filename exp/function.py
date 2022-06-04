import gzip
import os
import shutil
import sys

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

time = 0

@tick_execution_time
def lambda_handler():
    global time
    dst = 'result-' + str(time)
    src = 'compression-' + str(time)
    shutil.make_archive(dst, 'zip', src)
    time += 1

@mitosis_bench
def bench():
    lambda_handler()

if __name__ == '__main__':
    bench()
