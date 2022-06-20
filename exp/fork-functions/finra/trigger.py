import argparse
import socket

import zerorpc
# python trigger.py -child_hosts=val04 -process=2
parser = argparse.ArgumentParser()
parser.add_argument("-port", type=int, default=8080, help="rpc server port")
parser.add_argument("-child_hosts", type=str, default="localhost", help="rpc server host")
parser.add_argument("-process", type=int, default=1, help="rpc parallel num")
args = parser.parse_args()
port = args.port
process = args.process
child_hosts = str(args.child_hosts).split(' ')

master_cli = zerorpc.Client()
master_cli.connect("tcp://%s:%d" % ("127.0.0.1", 8090))

s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)

# start
master_cli.tick_rule_start()

for host in child_hosts:
    # Trigger without waiting
    for i in range(process):
        # print("send to %s:%d" % (host, port + i))
        s.sendto(b"data", (host, port + i))
