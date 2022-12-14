import os
import sys
import time

sys.path.append("../../common")  # include outer path

import syscall_lib
import bench
# import json
# import random
# import requests

import argparse
import mmap

parser = argparse.ArgumentParser()
parser.add_argument("-handler_id", type=int, default=73, help="rfork handler id")
parser.add_argument("-working_set", type=int, default=16777216,
                    help="working set size")
parser.add_argument("-exclude_execution", type=int, default=1,
                    help="Whether exclude the resume stage")
parser.add_argument("-profile", type=int, default=1, help="whether print out the profile data")
parser.add_argument("-pin", type=int, default=0, help="whether pin the descriptor in kernel")
parser.add_argument("-touch_ratio", type=int, default=100, help="child touch ratio")
parser.add_argument("-app_name", type=str, default="micro", help="application name")

args = parser.parse_args()

handler_id = args.handler_id
working_set = args.working_set
ret_imm = args.exclude_execution != 0
profile = args.profile
pin = args.pin
app_name = args.app_name
touch_ratio = args.touch_ratio
ret = ret_imm == 1

mm = mmap.mmap(-1, length=working_set)

def handler(working_sz):
    start = time.time()
    mm.seek(0)
    r = mm.read(working_sz)
    end = time.time()
    if profile == 1:
        bench.report("%s-execution with workingset %fMB" % (app_name, working_sz / (1024 * 1024)), start, end)


def prepare(key):
    fd = syscall_lib.open()
    start = time.time()
    if pin == 1:
        syscall_lib.call_prepare_ping(fd, key)
    else:
        syscall_lib.call_prepare(fd, key)
    end = time.time()
    if profile == 1:
        bench.report("%s-prepare" % app_name, start, end)


if __name__ == '__main__':
    handler(working_set)
    prepare(handler_id)
    if not ret_imm:
        handler(working_set * touch_ratio // 100)
    # print("done")
    os._exit(0)
