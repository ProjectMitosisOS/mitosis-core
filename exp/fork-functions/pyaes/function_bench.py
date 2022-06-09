from time import time
import random
import string
import pyaes
import json
import os
import sys

sys.path.append("../../common")  # include outer path
from func_bench_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'

def generate(length):
    letters = string.ascii_lowercase + string.digits
    return ''.join(random.choice(letters) for i in range(length))

def handler():
    """
    "params": [
                "{\"length_of_message\":\"22000\", \"num_of_iterations\":\"1\"}"
            ]
    :return:
    """
    length_of_message = 22000
    num_of_iterations = 1

    message = generate(length_of_message)

    # 128-bit key (16 bytes)
    KEY = b'\xa1\xf6%\x8c\x87}_\xcd\x89dHE8\xbf\xc9,'

    for loops in range(num_of_iterations):
        aes = pyaes.AESModeOfOperationCTR(KEY)
        ciphertext = aes.encrypt(message)

        aes = pyaes.AESModeOfOperationCTR(KEY)
        plaintext = aes.decrypt(ciphertext)
        aes = None

@func_exec_bench
def bench():
    handler()


if __name__ == '__main__':
    bench()
