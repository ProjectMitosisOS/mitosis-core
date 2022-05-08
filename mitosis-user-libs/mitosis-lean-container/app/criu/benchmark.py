import os
import mmap
import sys
from ctypes import sizeof
import argparse
import time

parser = argparse.ArgumentParser()
parser.add_argument("-run_sec", type=int, default=10, help="running seconds")
parser.add_argument("-working_set", type=int, default=16777216,
                    help="working set size")
parser.add_argument("-lock_file", type=str, default="lock", help="lock file")
parser.add_argument("-lock_string", type=str, default="0", help="check the lock file and compare with the first byte of lock string")

args = parser.parse_args()

run_sec = args.run_sec
working_set = args.working_set
lock_file = args.lock_file
lock_string = args.lock_string

mm = mmap.mmap(-1, length=working_set)
mm_lock = None
with open(lock_file, 'rb') as f:
    fd = f.fileno()
    mm_lock = mmap.mmap(fd, 0, flags=mmap.MAP_SHARED, prot=mmap.PROT_READ)

def touch_working_set(working_sz):
    mm.seek(0)
    mm.read(working_sz)

def wait():
    while chr(mm_lock[0]) == lock_string:
        pass

if __name__ == '__main__':
    # parent first touch
    print("%d" % (os.getpid()))
    touch_working_set(working_sz=working_set)
    # wait for being dumpped
    wait()
    cnt = 0
    print("counter %d" % cnt)
    touch_working_set(working_sz=working_set)

    # holding
    while cnt < run_sec:
        print("counter %d" % cnt)
        cnt += 1
        time.sleep(1)
