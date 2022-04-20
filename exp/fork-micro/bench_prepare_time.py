import mmap
import sys
from ctypes import sizeof

sys.path.append("../common")  # include outer path

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

# start = time.time()
# end = time.time()
time.sleep(1)

mm = mmap.mmap(-1, length=working_set)
# item = 1
# buffer = [item for _ in range(working_set // item.__sizeof__())]


def report(name):
    passed_us = (end - start) * 1000000
    print("[%s] time: %.2f us" % (str(name), passed_us))


def touch_working_set(working_sz):
    global start, end
    start = time.time()
    mm.seek(0)
    mm.read(working_sz)
    end = time.time()
    report("execution")


if __name__ == '__main__':
    global start, end

    fd = syscall_lib.open()
    # parent first touch
    touch_working_set(working_sz=working_set)
    # Call prepare
    cnt = 0
    print("counter %d" % cnt)
    start = time.time()
    syscall_lib.call_prepare(fd, handler_id)
    end = time.time()
    report("prepare")
