import argparse
import os
import time
from functools import wraps
import syscall_lib
import bench

parser = argparse.ArgumentParser()
parser.add_argument("-handler_id", type=int, default=73, help="rfork handler id")
parser.add_argument("-working_set", type=int, default=16777216,
                    help="working set size")
parser.add_argument("-exclude_execution", type=int, default=1,
                    help="Whether exclude the resume stage")
parser.add_argument("-profile", type=int, default=1, help="whether print out the profile data")
parser.add_argument("-pin", type=int, default=0, help="whether pin the descriptor in kernel")
parser.add_argument("-prepare",type=int, default=1, help="whether to call prepare")
parser.add_argument("-touch_ratio", type=int, default=100, help="child touch ratio")
parser.add_argument("-app_name", type=str, default="micro", help="application name")
args = parser.parse_args()

handler_id = args.handler_id
working_set = args.working_set
profile = args.profile
pin = args.pin
whether_prepare = args.prepare
app_name = args.app_name
touch_ratio = args.touch_ratio
ret_imm = args.exclude_execution != 0
ret = ret_imm == 1

def mitosis_bench(handler):
    """
    mitosis benchmark
        Usage
            @mitosis_bench
            def bench():
                my_handler()
    """

    def prepare(key):
        fd = syscall_lib.open()
        start = time.time()
        if pin == 1:
            syscall_lib.call_prepare_ping(fd, key)
        else:
            syscall_lib.call_prepare(fd, key)
        end = time.time()
        if profile == 1:
            # print("done...")
            bench.report("%s-prepare" % app_name, start, end)

    @wraps(handler)
    def wrapper(*args, **kwargs):
        print("cold start")
        result = handler(*args, **kwargs)

        print("warm start")
        result = handler(*args, **kwargs)

        if whether_prepare == 1:
            prepare(handler_id)
        if not ret_imm:
            result = handler(*args, **kwargs)
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
            bench.report("%s-execution" % app_name, start, end)
        return result

    return wrapper
