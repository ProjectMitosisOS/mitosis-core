import mmap
import os
import sys
from ctypes import sizeof

sys.path.append("../common")  # include outer path
from criu_wrapper import *

import syscall_lib
import argparse
import time

parser = argparse.ArgumentParser()
parser.add_argument("-working_set", type=int, default=16777216,
                    help="working set size")
parser.add_argument("-touch_ratio", type=int, default=100, help="child touch ratio")

args, _ = parser.parse_known_args()

touch_ratio = args.touch_ratio
working_set = args.working_set

mm = '1' * working_set

@tick_execution_time
def touch_working_set(working_sz):
    mm[0:working_sz-1]

@criu_bench
def bench():
    touch_working_set(working_sz=(working_set * touch_ratio // 100))

if __name__ == '__main__':
    bench()
