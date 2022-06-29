import socket
from PIL import Image
import json
import os
import sys

import uuid
from time import time
from PIL import Image, ImageFilter

server_addr = 'uds.socket'
socket_family = socket.AF_UNIX
socket_type = socket.SOCK_STREAM

TMP = '/dev/shm/'

def flip(image, file_name):
    path_list = []
    path = TMP + "flip-left-right-" + file_name
    img = image.transpose(Image.FLIP_LEFT_RIGHT)
    img.save(path)
    path_list.append(path)

    path = TMP + "flip-top-bottom-" + file_name
    img = image.transpose(Image.FLIP_TOP_BOTTOM)
    img.save(path)
    path_list.append(path)

    return path_list


def rotate(image, file_name):
    path_list = []
    path = TMP + "rotate-90-" + file_name
    img = image.transpose(Image.ROTATE_90)
    img.save(path)
    path_list.append(path)

    path = TMP + "rotate-180-" + file_name
    img = image.transpose(Image.ROTATE_180)
    img.save(path)
    path_list.append(path)

    path = TMP + "rotate-270-" + file_name
    img = image.transpose(Image.ROTATE_270)
    img.save(path)
    path_list.append(path)

    return path_list


def imgfilter(image, file_name):
    path_list = []
    path = TMP + "blur-" + file_name
    img = image.filter(ImageFilter.BLUR)
    img.save(path)
    path_list.append(path)

    path = TMP + "contour-" + file_name
    img = image.filter(ImageFilter.CONTOUR)
    img.save(path)
    path_list.append(path)

    path = TMP + "sharpen-" + file_name
    img = image.filter(ImageFilter.SHARPEN)
    img.save(path)
    path_list.append(path)

    return path_list


def gray_scale(image, file_name):
    path = TMP + "gray-scale-" + file_name
    img = image.convert('L')
    img.save(path)
    return [path]


def resize(image, file_name):
    path = TMP + "resized-" + file_name
    image.thumbnail((128, 128))
    image.save(path)
    return [path]

def image_processing(file_name, image_path):
    path_list = []
    start = time()
    with Image.open(image_path) as image:
        tmp = image
        # path_list += flip(image, file_name)
        path_list += rotate(image, file_name)

    latency = time() - start
    return latency, path_list

try:
    os.remove(server_addr)
except:
    pass

my_socket = socket.socket(socket_family, socket_type)
my_socket.bind(server_addr)
my_socket.listen(1)

connection, client_address = my_socket.accept()

def handler():
    in_key = 'test.jpeg'
    latency, path_list = image_processing(in_key, in_key)

while True:
    connection.recv(1)
    handler()
    connection.sendall('\0')
