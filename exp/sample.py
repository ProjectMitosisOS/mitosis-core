import mmap
import os
import sys
from ctypes import sizeof

sys.path.append("common")  # include outer path

import syscall_lib
import argparse
import time

parser = argparse.ArgumentParser()
parser.add_argument("-handler_id", type=int, default=73, help="rfork handler id")
parser.add_argument("-run_sec", type=int, default=10, help="running seconds")
parser.add_argument("-working_set", type=int, default=16777216,
                    help="working set size")

args = parser.parse_args()

handler_id = args.handler_id
run_sec = args.run_sec
working_set = args.working_set

time.sleep(1)
mm = mmap.mmap(-1, length=working_set)


def touch_working_set(working_sz):
    start = time.time()
    mm.seek(0)
    mm.read(working_sz)


if __name__ == '__main__':
    cnt = 0
    print("counter %d" % cnt)
    fd = syscall_lib.open()

    # parent first touch
    touch_working_set(working_sz=working_set)

    # Call prepare
    syscall_lib.call_prepare(fd, handler_id)

    touch_working_set(working_sz=working_set)
    print("finish")
    # holding
    while cnt < run_sec:
        print("counter %d" % cnt)
        cnt += 1
        time.sleep(1)
    os._exit(0)