import os
import subprocess
import sys
import time

sys.path.append("../../common")  # include outer path

import syscall_lib
import bench

## Migration related
import argparse

parser = argparse.ArgumentParser()
parser.add_argument("-handler_id", type=int, default=73, help="rfork handler id")
parser.add_argument("-exclude_execution", type=int, default=1,
                    help="Whether exclude the resume stage")
parser.add_argument("-profile", type=int, default=1, help="whether print out the profile data")
parser.add_argument("-pin", type=int, default=0, help="whether pin the descriptor in kernel")
parser.add_argument("-app_name", type=str, default="micro", help="application name")

args = parser.parse_args()

handler_id = args.handler_id
ret_imm = args.exclude_execution != 0
profile = args.profile
pin = args.pin
app_name = args.app_name
ret = ret_imm == 1

start = time.time()
end = time.time()

## Migration related end

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'

tmp = '/tmp/'
bs = 'bs=4M'
count = 'count=100'


def handler():
    global start, end
    start = time.time()

    out_fd = open(tmp + 'io_write_logs', 'w')
    dd = subprocess.Popen(['dd', 'if=/dev/zero', 'of=/dev/zero', bs, count], stderr=out_fd)
    dd.communicate()

    subprocess.check_output(['ls', '-alh', tmp])

    end = time.time()
    if profile == 1:
        bench.report("%s-execution" % app_name, start, end)


def prepare(key):
    global start, end
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
    handler()
    prepare(handler_id)
    if not ret_imm:
        handler()
    os._exit(0)
