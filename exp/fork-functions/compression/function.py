import gzip
import os
import sys

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'


@tick_execution_time
def lambda_handler():
    file_size = 256 * 1024
    file_write_path = 'uncompressed'
    with open(file_write_path, 'wb') as f:
        f.write(os.urandom(file_size))

    with open(file_write_path, 'rb') as f:
        with gzip.open('compressed.gz', 'wb') as gz:
            gz.writelines(f)


@mitosis_bench
def bench():
    lambda_handler()


if __name__ == '__main__':
    bench()
