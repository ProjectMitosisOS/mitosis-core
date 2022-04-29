from PIL import Image
import json
import os
import sys

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'


## Migration related end
def init():
    im = Image.open('test.jpg')
    size = (128, 128)
    im.thumbnail(size)
    im.close()
    del (im)
    del (size)


@tick_execution_time
def handler():
    im = Image.open('test.jpg')
    size = (128, 128)
    im.thumbnail(size)
    im.save('thumbnail.jpg')


@mitosis_bench
def bench():
    handler()


if __name__ == '__main__':
    init()
    bench()
