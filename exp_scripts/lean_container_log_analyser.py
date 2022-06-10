import argparse
import os
import re
import time
from subprocess import PIPE, Popen

import numpy as np

arg_parser = argparse.ArgumentParser(
    description=''' Log analyser for running the cluster''')
arg_parser.add_argument(
    '-i', '--input', default="out", type=str,
    help='The input directory')

arg_parser.add_argument(
    '--xfactor', default=[], nargs='+',
    help='x-axis multi-factor')

args = arg_parser.parse_args()
input_dir = args.input
x_factor = len(args.xfactor)

def parse_line(line):
    machine_pattern = re.compile(r"^@(.*?)\s.*?$")
    machine_process = re.compile(r"^.*?\[(.*?)]\s.*?$")

    thpt_pattern = re.compile(r"^.*?Throughput:\s(.*?)\scontainers.*?$")

    lat_pattern = re.compile(r"^.*?latency\s(.*?)\sms")
    if thpt_pattern.match(line) is None:
        return None, None, None
    # machine = machine_pattern.findall(line)[-1]
    process_name = machine_process.findall(line)[-1]
    thpt = float(thpt_pattern.findall(line)[-1])
    lat = float(lat_pattern.findall(line)[-1])
    return process_name, thpt, lat


def analyse(file_path):
    with open(file_path) as f:
        thpt_hash = {}
        lat_hash = {}
        for line in f:
            machine, thpt, lat = parse_line(line)
            if machine is None: continue
            if machine not in thpt_hash.keys():
                thpt_hash[machine] = []
            if machine not in lat_hash.keys():
                lat_hash[machine] = []
            thpt_hash[machine].append(thpt)
            lat_hash[machine].append(lat)

        thpt_avg = {}
        lat_avg = {}
        for machine, thpts in thpt_hash.items():
            thpt_avg[machine] = np.mean(np.sort(thpts)[0:-10])
        for machine, lats in lat_hash.items():
            lat_avg[machine] = np.mean(np.sort(lats)[10:-1])

    sum_up_thpt = sum(list(thpt_avg.values()))
    avg_lat = np.mean(list(lat_avg.values()))
    return sum_up_thpt, avg_lat


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
            out, exitcode, err = '', 128, 'timeout'
            return out, exitcode, err
        time.sleep(0.1)




def trigger(dictory):
    from os.path import join, getsize

    for root, dirs, files in os.walk(dictory):
        dic = {}
        for f in files:
            pattern = re.compile(r"^run-.*?toml$")
            if pattern.match(f) is not None:
                log_path = "{}/{}.txt".format(dictory, f)
                thpt, lat = analyse(log_path)
                print('{}: thpt: {} op/s\t\tlatency: {} us'.format(f, thpt, lat))
                dic[f] = {'thpt': thpt, 'lat': lat}



if __name__ == '__main__':
    trigger(input_dir)
