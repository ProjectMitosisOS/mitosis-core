import argparse
import os
import re
import signal
import time
from subprocess import PIPE, Popen

arg_parser = argparse.ArgumentParser(
    description=''' Log analyser for running the cluster''')
arg_parser.add_argument(
    '-i', '--input', default="out", type=str,
    help='The input directory')

arg_parser.add_argument(
    '-a', '--arguments', default="", type=str,
    help='Running flag of bootstrap')

args = arg_parser.parse_args()
input_dir = args.input
arguments = args.arguments


def sys_command_outstatuserr(cmd, timeout=30):
    p = Popen(cmd, stdout=PIPE, stderr=PIPE, shell=True)
    t_beginning = time.time()
    seconds_passed = 0
    while True:
        if p.poll() is not None:
            res = p.communicate()
            exitcode = p.poll() if p.poll() else 0
            return res[0], exitcode, res[1]
        seconds_passed = time.time() - t_beginning
        if timeout and seconds_passed > timeout:
            p.terminate()
            os.kill(p.pid, signal.SIGINT)
            out, exitcode, err = '', 128, 'timeout'
            return out, exitcode, err
        time.sleep(0.1)


def trigger_bootstrap(dictory):
    for root, dirs, files in os.walk(dictory):
        for f in files:
            pattern = re.compile(r"^run.*?toml$")
            # pattern = re.compile(r"^run48.toml$")
            if pattern.match(f) is not None:
                log_path = "{}/{}.txt".format(dictory, f)
                cmd = "python bootstrap.py -f {} {} > {}".format(
                    os.path.join(root, f), str(arguments), log_path)
                sys_command_outstatuserr(cmd, timeout=60)
                print('finish', f)


if __name__ == '__main__':
    trigger_bootstrap(input_dir)
