import argparse
import os
import time
from functools import wraps
import syscall_lib
import bench
import mmap

parser = argparse.ArgumentParser()
parser.add_argument("-profile", type=int, default=1, help="whether print out the profile data")
parser.add_argument("-app_name", type=str, default="micro", help="application name")
parser.add_argument("-lock_file", type=str, default="lock", help="lock file")
parser.add_argument("-lock_string", type=str, default="0", help="check the lock file and compare with the first byte of lock string")
parser.add_argument("-exclude_execution", type=int, default=0,
                    help="Whether exclude the resume stage")
args, _ = parser.parse_known_args()

profile = args.profile
app_name = args.app_name
lock_file = args.lock_file
lock_string = args.lock_string
ret_imm = args.exclude_execution != 0

mm_lock = None

def criu_bench(handler):
    def wait():
        with open(lock_file, 'rb') as f:
            fd = f.fileno()
            mm_lock = mmap.mmap(fd, 0, flags=mmap.MAP_SHARED, prot=mmap.PROT_READ)
        while chr(mm_lock[0]) == lock_string:
            pass

    @wraps(handler)
    def wrapper(*args, **kwargs):
        handler(*args, **kwargs)
        wait()
        if not ret_imm:
            handler(*args, **kwargs)
        os._exit(0)

    return wrapper

def tick_execution_time(handler):
    """

    """
    @wraps(handler)
    def wrapper(*args, **kwargs):
        start = time.time()
        result = handler(*args, **kwargs)
        end = time.time()
        if profile == 1:
            print("before start python handler: %f" % (start))
            bench.report("%s-execution" % app_name, start, end)
        return result

    return wrapper

