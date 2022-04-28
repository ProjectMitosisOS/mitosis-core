import os
import random
import string
import sys
import time
import pyaes

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'
# from numpy import random

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


def generate(length):
    letters = string.ascii_lowercase + string.digits
    return ''.join(random.choice(letters) for i in range(length))


## Migration related end

def handler():
    start = time.time()

    ## Body start
    length_of_message = 256
    num_of_iterations = 10

    message = generate(length_of_message)

    KEY = b'\xa1\xf6%\x8c\x87}_\xcd\x89dHE8\xbf\xc9,'

    for loops in range(num_of_iterations):
        aes = pyaes.AESModeOfOperationCTR(KEY)
        ciphertext = aes.encrypt(message)
        # print(ciphertext)

        aes = pyaes.AESModeOfOperationCTR(KEY)
        plaintext = aes.decrypt(ciphertext)
        # print(plaintext)
        aes = None

    ## Body end
    end = time.time()
    if profile == 1:
        bench.report("%s-execution" % app_name, start, end)


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
    handler()
    prepare(handler_id)
    if not ret_imm:
        handler()
    os._exit(0)
