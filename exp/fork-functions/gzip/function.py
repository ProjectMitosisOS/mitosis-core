import gzip
import os
import sys
import time

sys.path.append("../../common")  # include outer path

import syscall_lib
import bench

## Migration related
app_name = "gzip"

import argparse

parser = argparse.ArgumentParser()
parser.add_argument("-handler_id", type=int, default=103, help="rfork handler id")
parser.add_argument("-profile", type=int, default=1, help="whether print out the profile data")
parser.add_argument("-pin", type=int, default=0, help="whether pin the descriptor in kernel")
args = parser.parse_args()

handler_id = args.handler_id
profile = args.profile
pin = args.pin

## Migration related end

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'

def prepare():
    pass


def handler():
    global start, end
    start = time.time()

    ## Body
    file_size = 4 * 1024 * 1024
    file_write_path = 'uncompressed'
    with open(file_write_path, 'wb') as f:
        f.write(os.urandom(file_size))

    with open(file_write_path, 'rb') as f:
        with gzip.open('compressed.gz', 'wb') as gz:
            gz.writelines(f)
    ## Body end

    end = time.time()
    if profile == 1:
        bench.report("%s-execution" % app_name, start, end)


def checkpoint(key):
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
    checkpoint(handler_id)
    handler()
    os._exit(0)
