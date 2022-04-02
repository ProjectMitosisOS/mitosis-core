import time
import os
import syscall_lib

fd = -1 
counter = 0

def main():
    global fd, counter
    fd = syscall_lib.open()

    time.sleep(1)
    print("open MITOSIS client, fd {}", fd)

    syscall_lib.call_prepare(fd, 73)

    while True:
        time.sleep(1)
        counter += 1
        s = "check counter %d, fd %d" % (counter,fd)
        print(s)    

if __name__ == '__main__':
    main()
