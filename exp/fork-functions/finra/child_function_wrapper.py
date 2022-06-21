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


def runcmd(command):
    cmd = "%s -port=%d" % (command, 8080 + loop)
    proc = subprocess.Popen(cmd,
                            shell=True,
                            stdout=subprocess.PIPE)
    out, err = proc.communicate()
    master_cli.report_finish_event()
    print(out)


if __name__ == '__main__':
    runcmd(command=cmd)
