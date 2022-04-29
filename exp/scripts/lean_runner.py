import os
import re

import numpy as np

import numpy as np


def execCmd(cmd):
    r = os.popen(cmd)
    text = r.read()
    r.close()
    return text


def analysis_log(file_name, cmd_txt):
    qps_pattern = re.compile(r"^.*?qps\s(.*?)\s.*?$")
    latency_pattern = re.compile(r"^.*?latency\s(.*?)\sms$")

    qps_li, lat_li = [], []
    for line in cmd_txt.split('\n'):
        if qps_pattern.match(line) is not None:
            qps = float(qps_pattern.findall(line)[-1])
            lat = float(latency_pattern.findall(line)[-1])
            qps_li.append(qps)
            lat_li.append(lat)
    print("Execute on file %s\tqps %f op/s\tlatency %f ms" % (file_name, np.mean(qps_li), np.mean(lat_li)))


def trigger_bootstrap(boost_path, dictorys):
    from os.path import join, getsize

    for dictory in dictorys:
        again = True  # Do exp again
        # again = False       # Just get result
        for root, dirs, files in os.walk(dictory):
            for f in files:
                # pattern = re.compile(r"^.*?helloworld.*?toml$")
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
                          # micro startup
                          # "/Users/lufangming/Documents/repos/mitosis/scripts/exp/fork-micro/startup",

                          # touch ratio
                          "/Users/lufangming/Documents/repos/mitosis/scripts/exp/fork-micro/touch-ratio",

                          # functions
                          # "/Users/lufangming/Documents/repos/mitosis/scripts/exp/fork-micro-function/startup"
                      ]
                      )

    # print(analyse('run/run-peak.toml.txt'))
