import argparse
import socket

# python trigger.py -child_hosts=val04 -process=2
import time

parser = argparse.ArgumentParser()
parser.add_argument("-child_hosts", type=str, default="", help="rpc server host")
parser.add_argument("-parent_host", type=str, default="localhost", help="parent host")
parser.add_argument("-process", type=int, default=1, help="rpc parallel num")
args = parser.parse_args()
process = args.process
parent_host = args.parent_host
child_hosts = str(args.child_hosts).split(' ')
master_port = 7000
parent_port = 8000
trigger_port = 9000

s_tcp = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s_udp = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s_udp.sendto(str(1 + len(child_hosts) * process).encode(), ('', master_port))

s_tcp.connect((parent_host, parent_port))
s_tcp.sendall(parent_host.encode())

s_tcp.recv(1024).decode()

for host in child_hosts:
    # Trigger without waiting
    for i in range(process):
        s_udp.sendto(b"data", (host, parent_port + i))
s_tcp.close()
s_udp.close()
