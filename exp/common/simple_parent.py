import syscall_lib
import argparse
import time

parser = argparse.ArgumentParser()
parser.add_argument("-handler_id", type=int, default=73,
                    help="rfork handler id")
args = parser.parse_args()
sys.path.append("module_path")

handler_id = args.handler_id

print("handler id %d" % handler_id)

if __name__ == '__main__':
    fd = syscall_lib.open()
    counter = 0

    time.sleep(1)
    print("open MITOSIS client, fd {}", fd)
    syscall_lib.call_prepare(fd, handler_id)

    while True:
        counter += 1
        s = "check counter %d, fd %d" % (counter, fd)
        print(s)
        time.sleep(1)
