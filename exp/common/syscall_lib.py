import fcntl
import os

PREPARE = 4
RESUME_LOCAL = 5
RESUME_LOCAL_RPC = 6

def open():
    fd = os.open('/dev/mitosis-syscalls', os.O_RDWR)
    return fd

def call_prepare(sd, key):
    res = fcntl.ioctl(sd, PREPARE, key)
    return res

def call_resume_local(sd,key):
    res = fcntl.ioctl(sd, RESUME_LOCAL, key)
    return res

def call_resume_local_rpc(sd,key):
    res = fcntl.ioctl(sd, RESUME_LOCAL_RPC, key)
    return res
