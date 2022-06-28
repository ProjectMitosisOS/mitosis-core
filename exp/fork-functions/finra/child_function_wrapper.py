import os
import socket
import subprocess
import argparse

import zerorpc

parser = argparse.ArgumentParser()
parser.add_argument("-command", type=str, default="ls", help="running cmd")
parser.add_argument("-master_host", type=str, default="localhost", help="host name of master")
parser.add_argument("-loop", type=int, default=0, help="loop num")
args = parser.parse_args()

cmd = args.command
master_host = args.master_host
loop = args.loop

master_cli = zerorpc.Client()
master_cli.connect("tcp://%s:%d" % (master_host, 8090))

s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.bind(("0.0.0.0", 8080 + loop))
data, addr = s.recvfrom(1024)


def runcmd(command):
    cmd = command
    proc = subprocess.Popen(cmd, shell=True)
    proc.wait()
    master_cli.report_finish_event()


if __name__ == '__main__':
    runcmd(command=cmd)
