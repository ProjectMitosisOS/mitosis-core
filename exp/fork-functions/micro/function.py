import os

import syscall_lib
import argparse
import mmap

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'

parser = argparse.ArgumentParser()
parser.add_argument("-handler_id", type=int, default=73, help="rfork handler id")
parser.add_argument("-working_set", type=int, default=16777216,
                    help="working set size")
parser.add_argument("-ret_imm", type=int, default=1,
                    help="If only benchmark on the startup time")

args = parser.parse_args()

handler_id = args.handler_id
working_set = args.working_set
ret_imm = args.ret_imm

mm = mmap.mmap(-1, length=working_set)


def handler(working_sz):
    mm.seek(0)
    mm.read(working_sz)


def checkpoint(key):
    fd = syscall_lib.open()
    syscall_lib.call_prepare_ping(fd, key)  # we should ping here


if __name__ == '__main__':
    ret = ret_imm == 1
    handler(working_set)
    checkpoint(handler_id)
    if not ret:
        handler(working_set)
    os._exit(0)