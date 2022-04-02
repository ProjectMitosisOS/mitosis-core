import fcntl
import os

PREPARE = 4  ## TODO, we need to import these constants from MITOSIS-protocol
RESUME_LOCAL = 5 

def open():
    fd = os.open('/dev/mitosis-syscalls', os.O_RDWR)
    return fd

def call_prepare(sd, key):
    res = fcntl.ioctl(sd, PREPARE, key)
    return res

def call_resume_local(sd,key):
    res = fcntl.ioctl(sd, RESUME_LOCAL, key)
    return res    