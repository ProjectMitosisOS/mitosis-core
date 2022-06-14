from PIL import Image
import json
import os
import sys

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'
import uuid
from time import time
from PIL import Image, ImageFilter

TMP = '/tmp/'

def resize(image, file_name):
    path = TMP + "resized-" + file_name
    image.thumbnail((128, 128))
    image.save(path)
    return [path]

def image_processing(file_name, image_path):
    path_list = []
    start = time()
    with Image.open(image_path) as image:
        path_list += resize(image, file_name)

    latency = time() - start
    return latency, path_list

@tick_execution_time
def handler():
    in_key = 'test.jpg'
    latency, path_list = image_processing(in_key, in_key)

@mitosis_bench
def bench():
#    print("before image processing")    
    handler()
#    print("image processing done") 

if __name__ == '__main__':
    bench()