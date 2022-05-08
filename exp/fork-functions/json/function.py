import json
import os
import sys

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'

f = open('linux.json')
content = f.read()


@tick_execution_time
def lambda_handler():
    json_data = json.loads(content)
    str_json = json.dumps(json_data, indent=4)


@mitosis_bench
def bench():
    lambda_handler()


if __name__ == '__main__':
    bench()
