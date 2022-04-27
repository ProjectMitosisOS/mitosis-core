import os
import sys
import time

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'
import cv2

sys.path.append("../../common")  # include outer path

import syscall_lib
import bench

## Migration related
app_name = "video-processing"
import argparse

parser = argparse.ArgumentParser()
parser.add_argument("-handler_id", type=int, default=101, help="rfork handler id")
parser.add_argument("-profile", type=int, default=1, help="whether print out the profile data")
parser.add_argument("-pin", type=int, default=0, help="whether pin the descriptor in kernel")
args = parser.parse_args()

handler_id = args.handler_id
profile = args.profile
pin = args.pin

## Migration end

tmp = "/tmp/"
FILE_NAME_INDEX = 0
FILE_PATH_INDEX = 2

dump_key = 73


def video_processing(object_key, video_path):
    # file_name = object_key.split(".")[FILE_NAME_INDEX]
    file_name = 'test'
    result_file_path = tmp + file_name + '-output.avi'

    if os.path.exists(result_file_path):
        os.remove(result_file_path)

    video = cv2.VideoCapture(video_path)

    width = int(video.get(3))
    height = int(video.get(4))

    fourcc = cv2.VideoWriter_fourcc(*'XVID')
    out = cv2.VideoWriter(result_file_path, fourcc, 20.0, (width, height))

    while video.isOpened():
        ret, frame = video.read()

        if ret:
            gray_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
            tmp_file_path = tmp + 'tmp.jpg'
            cv2.imwrite(tmp_file_path, gray_frame)
            gray_frame = cv2.imread(tmp_file_path)
            out.write(gray_frame)
        else:
            break

    video.release()
    out.release()
    return result_file_path


def prepare():
    pass


def handler():
    global start, end
    start = time.time()

    object_key = 'same_video.test.mp4'
    download_path = 'SampleVideo_1280x720_10mb.mp4'

    upload_path = video_processing(object_key, download_path)

    end = time.time()
    if profile == 1:
        bench.report("%s-execution" % app_name, start, end)


def checkpoint(key):
    global start, end
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
    prepare()
    handler()
    checkpoint(handler_id)
    handler()
    os._exit(0)
