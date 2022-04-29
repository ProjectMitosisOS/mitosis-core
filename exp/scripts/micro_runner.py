import os
import re

import numpy as np


def execCmd(cmd):
    r = os.popen(cmd)
    text = r.read()
    r.close()
    return text


def analysis_log(file_name, cmd_txt):
    prepare_time_pattern = re.compile(r"^.*?prepare]\stime:\s(.*?)\sus")
    execute_time_pattern = re.compile(r"^.*?execution]\stime:\s(.*?)\sus")
    # print(cmd_txt)
    prepare_li = []
    exe_li = []
    for line in cmd_txt.split('\n'):
        if prepare_time_pattern.match(line) is not None:
            prepare = float(prepare_time_pattern.findall(line)[-1])
            prepare_li.append(prepare)
        if execute_time_pattern.match(line) is not None:
            exe = float(execute_time_pattern.findall(line)[-1])
            exe_li.append(exe)
    prepare = prepare_li[0]

    print("Execute on file %s, prepare latency \t\t%f us" % (file_name, prepare))
    for i, exe_lat in enumerate(exe_li):
        print("Execute on file %s, %d-th touch latency \t%f us" % (file_name, i, exe_lat))


def trigger_bootstrap(boost_path, dictorys):
    from os.path import join, getsize

    for dictory in dictorys:
        for root, dirs, files in os.walk(dictory):
            for f in files:
                # pattern = re.compile(r"run.toml$")
                pattern = re.compile(r"^run.*?toml$")
                if pattern.match(f) is not None:
                    log_path = "{}/{}.txt".format(dictory, f)
                    cmd = "python {} -f {}".format(boost_path, os.path.join(root, f))
                    cmd_txt = execCmd(cmd)
                    analysis_log(f, cmd_txt)
                    print("=======================================================================================")


if __name__ == '__main__':
    trigger_bootstrap('bootstrap.py',
                      [
                          # '/Users/lufangming/Documents/repos/mitosis/scripts/exp/fork-micro/prepare-execution',
                          '/Users/lufangming/Documents/repos/mitosis/scripts/exp/fork-micro-function/prepare-execution'
                          # '/Users/lufangming/Documents/repos/mitosis/scripts'
                      ]
                      )
