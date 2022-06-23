import argparse
import socket

import zerorpc

# python trigger.py -child_hosts=val04 -process=2
parser = argparse.ArgumentParser()
parser.add_argument("-port", type=int, default=8080, help="rpc server port")
parser.add_argument("-child_hosts", type=str, default="", help="rpc server host")
parser.add_argument("-parent_host", type=str, default="localhost", help="parent host")
parser.add_argument("-process", type=int, default=1, help="rpc parallel num")
args = parser.parse_args()
port = args.port
process = args.process
parent_host = args.parent_host
child_hosts = str(args.child_hosts).split(' ')

master_cli = zerorpc.Client()
master_cli.connect("tcp://%s:%d" % ("127.0.0.1", 8090))

s_tcp = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
s_udp = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

s_tcp.connect(("localhost", 6000))
# start
master_cli.tick_rule_start(process * len(child_hosts))

s_tcp.sendall(b"data")
s_tcp.recv(1024).decode()

if args.child_hosts == '':
    master_cli.report_finish_event()
else:
    for host in child_hosts:
        # Trigger without waiting
        for i in range(process):
            s_udp.sendto(b"data", (host, port + i))
s_tcp.close()
s_udp.close()
