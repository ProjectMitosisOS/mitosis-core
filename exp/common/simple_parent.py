import os
import sys

import syscall_lib
import argparse
import time

parser = argparse.ArgumentParser()
parser.add_argument("-handler_id", type=int, default=73,
                    help="rfork handler id")
parser.add_argument("-pin", type=int, default=73,
                    help="whether pin in kernel")
args = parser.parse_args()
sys.path.append("module_path")

handler_id = args.handler_id
pin = args.pin

if __name__ == '__main__':
    fd = syscall_lib.open()

    print("open MITOSIS client, fd {}", fd)
    time.sleep(1)
    counter = 0
    if pin != 0:
        syscall_lib.call_prepare_ping(fd, handler_id)
        for i in range(5):
            counter += 1
            s = "check counter %d, fd %d" % (counter, fd)
            print(s)
            time.sleep(1)
        os._exit(0)
    else:
        syscall_lib.call_prepare(fd, handler_id)
        while True:
            counter += 1
            s = "check counter %d, fd %d" % (counter, fd)
            print(s)
            time.sleep(1)
