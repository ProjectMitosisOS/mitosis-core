import mmap
import os
import sys
from ctypes import sizeof

sys.path.append("../common")  # include outer path
from criu_wrapper import *

import syscall_lib
import argparse
import time


# Define a function to return the current datetime
def get_timestamp():
    from datetime import datetime
    return str(datetime.now())

@tick_execution_time
def handler():
    print("hello world")

@criu_bench
def bench():
    handler()
    
if __name__ == '__main__':
    bench()
