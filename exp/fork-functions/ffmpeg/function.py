import os
import re
import subprocess
import sys

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'

cleanup_re = re.compile('[^a-z]+')
tmp = '/dev/shm/'


@tick_execution_time
def lambda_handler():
    subprocess.check_output(['/usr/local/ffmpeg/bin/ffmpeg', '-y',
                             '-i', '/tmp/test.mp4',
                             '-vf', 'hflip', '/tmp/out.mp4'])


@mitosis_bench
def bench():
    lambda_handler()


if __name__ == '__main__':
    bench()
