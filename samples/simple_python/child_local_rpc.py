import time

import os
import syscall_lib

dump_key = 73

def main():
    fd = syscall_lib.open()
    print('fd: %d' % fd)
    syscall_lib.call_resume_local_rpc(fd, dump_key)

    while True:
        assert False

if __name__ == '__main__':
    main()
