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
    for line in cmd_txt.split('\n'):
        # if prepare_time_pattern.match(line) is not None:
        #     lat = float(prepare_time_pattern.findall(line)[-1])
        #     print("Execute on file %s , prepare latency %f us" % (file_name, lat))
        if execute_time_pattern.match(line) is not None:
            lat = float(execute_time_pattern.findall(line)[-1])
            print("Execute on file %s , execution latency %f us" % (file_name, lat))


def trigger_bootstrap(boost_path, dictorys):
    from os.path import join, getsize

    for dictory in dictorys:
        again = True  # Do exp again
        # again = False       # Just get result
        for root, dirs, files in os.walk(dictory):
            for f in files:
                # pattern = re.compile(r"^run-128M.*?toml$")
                pattern = re.compile(r"^run.*?.toml$")
                if pattern.match(f) is not None:
                    log_path = "{}/{}.txt".format(dictory, f)
                    cmd = "python {} -f {}".format(boost_path, os.path.join(root, f))
                    cmd_txt = execCmd(cmd)
                    analysis_log(f, cmd_txt)
                    # cmd = "python {} -f {} > {}".format(boost_path, os.path.join(root, f), log_path)
                    # os.system(cmd)
        print("=======================================================================================")


if __name__ == '__main__':
    trigger_bootstrap('bootstrap.py',
                      [

                          # prepare_time [CoW & Self-copy]
                          # '/Users/lufangming/Documents/repos/mitosis/scripts/exp/fork-micro/prepare',
                          # execute time
                          '/Users/lufangming/Documents/repos/mitosis/scripts/exp/fork-micro/execute',
                      ]
                      )

    # print(analyse('run/run-peak.toml.txt'))
