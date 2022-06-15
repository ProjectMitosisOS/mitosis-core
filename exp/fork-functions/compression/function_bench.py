import gzip
import os
import shutil
import sys
import time

sys.path.append("../../common")  # include outer path
from func_bench_wrapper import *


def handler():
    dst = 'result-0'
    src = 'compression-0'
    shutil.make_archive(dst, 'zip', src)
@func_exec_bench
def bench():
    handler()

if __name__ == '__main__':
    bench()
