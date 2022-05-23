import argparse
import os.path
import re

import toml

arg_parser = argparse.ArgumentParser(
    description=''' Benchmark generator for running the cluster''')
arg_parser.add_argument(
    '-f', metavar='CONFIG', dest='config', default="template.toml", type=str,
    help='The template file for generation')
arg_parser.add_argument(
    '-o', '--out', default="out", type=str,
    help='The output directory')
arg_parser.add_argument(
    '-d', default="{'placeholder':{}, 'hosts':{'role':{}}}", type=str)

args = arg_parser.parse_args()
args_kv = dict(args._get_kwargs())
template_dict = dict(eval(args_kv['d']))


def keyword(placeholder):
    target_key = ""
    for k, v in dict(placeholder).items():
        if type(v) is list:
            target_key = k
            break
    return target_key


def fill_placeholder(cmd, placeholder):
    res = []
    target_key = keyword(placeholder)
    placeholder_pattern = re.compile(r".*?\$\{(.*?)}.*?")
    places = re.findall(placeholder_pattern, cmd)
    target_holder_list = list(placeholder[target_key])
    for item in target_holder_list:
        c = str(cmd)
        for p in places:
            target = '${%s}' % p
            if '@' in target: continue

            if p != target_key:
                c = c.replace(target, str(placeholder[p]))
            else:
                c = c.replace(target, str(item))
        res.append(c)
    return (res, target_holder_list)


def get_intersection(dic, upper_dict):
    res = upper_dict.copy()
    for k, v in dict(dic).items():
        if k not in res.keys():
            res[k] = v
    return res


def handle_template(config):
    out_dir = args.out
    if not os.path.isdir(out_dir):
        print("creating toml output dir", out_dir)
        os.mkdir(out_dir)

    tem = dict(config['template'])
    # merge into the template
    placeholder = get_intersection(tem['placeholder'], template_dict['placeholder'])
    keyword_place = keyword(placeholder)
    cmd_mat = []  # item_cnt * file_cnt
    name_mat = []
    for item in tem['pass']:
        cc, names = fill_placeholder(item['cmd'], placeholder)

        cmd_mat.append(cc)
        name_mat.append(names)

    for (j, key) in enumerate(placeholder[keyword_place]):
        fname = '{}/run-{}.toml'.format(str(out_dir), str(key))
        out_dict = get_intersection(dict(config['global']).copy(), template_dict)
        out_dict['pass'] = []
        for (i, item) in enumerate(tem['pass']):
            cmd = cmd_mat[i][j]
            item['host'] = template_dict['hosts'][item['role']] if item['role'] in template_dict['hosts'].keys() else \
                item['host']
            for (k, host) in enumerate(item['host']):
                key = '${@incr}'
                out_dict['pass'].append({
                    'host': host, 'path': template_dict['path'] + '/' + item['path'],
                    'cmd': cmd if key not in cmd else cmd.replace(key, str(1 + k))
                })
        with open(fname, 'w') as f:
            toml.dump(out_dict, f)
            # print("generate toml config file", fname)


def main():
    config = toml.load(args.config)

    handle_template(config)


if __name__ == '__main__':
    main()
