import os
import sys
import time

from PIL import Image

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'

sys.path.append("../../common")  # include outer path

import syscall_lib
import bench

## Migration related
app_name = "image-processing"

import argparse

parser = argparse.ArgumentParser()
parser.add_argument("-handler_id", type=int, default=101, help="rfork handler id")
parser.add_argument("-profile", type=int, default=1, help="whether print out the profile data")
parser.add_argument("-pin", type=int, default=0, help="whether pin the descriptor in kernel")
args = parser.parse_args()

handler_id = args.handler_id
profile = args.profile
pin = args.pin

def prepare():
    im = Image.open('test.jpg')
    size = (128, 128)
    im.thumbnail(size)
    im.close()
    del(im)
    del(size)

## Migration related end

def handler():
    global start, end
    start = time.time()

    ## Body start
    im = Image.open('test.jpg')
    size = (128, 128)
    im.thumbnail(size)
    im.save('thumbnail.jpg')
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
    prepare()
    handler()
    checkpoint(handler_id)
    handler()
    os._exit(0)
