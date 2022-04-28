import os
import sys
import time

sys.path.append("../../common")  # include outer path
import syscall_lib
import bench

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'
from chameleon import PageTemplate
import six

## Migration related
import argparse

parser = argparse.ArgumentParser()
parser.add_argument("-handler_id", type=int, default=73, help="rfork handler id")
parser.add_argument("-exclude_execution", type=int, default=1,
                    help="Whether exclude the resume stage")
parser.add_argument("-profile", type=int, default=1, help="whether print out the profile data")
parser.add_argument("-pin", type=int, default=0, help="whether pin the descriptor in kernel")
parser.add_argument("-app_name", type=str, default="micro", help="application name")

args = parser.parse_args()

handler_id = args.handler_id
ret_imm = args.exclude_execution != 0
profile = args.profile
pin = args.pin
app_name = args.app_name
ret = ret_imm == 1

## Migration related end
BIGTABLE_ZPT = """\
<table xmlns="http://www.w3.org/1999/xhtml"
xmlns:tal="http://xml.zope.org/namespaces/tal">
<tr tal:repeat="row python: options['table']">
<td tal:repeat="c python: row.values()">
<span tal:define="d python: c + 1"
tal:attributes="class python: 'column-' + %s(d)"
tal:content="python: d" />
</td>
</tr>
</table>""" % six.text_type.__name__


tmpl = PageTemplate(BIGTABLE_ZPT)

def handler():
    start = time.time()

    num_of_rows = 4
    num_of_cols = 4

    tmpl = PageTemplate(BIGTABLE_ZPT)
    data = {}
    for i in range(num_of_cols):
        data[str(i)] = i

    table = [data for x in range(num_of_rows)]
    options = {'table': table}
    data = tmpl.render(options=options)
    end = time.time()
    if profile == 1:
        bench.report("%s-execution" % app_name, start, end)


def prepare(key):
    fd = syscall_lib.open()
    start = time.time()
    if pin == 1:
        print("pin to the kernel")
        syscall_lib.call_prepare_ping(fd, key)
    else:
        syscall_lib.call_prepare(fd, key)
    end = time.time()
    if profile == 1:
        bench.report("%s-prepare" % app_name, start, end)


if __name__ == '__main__':
    handler()
    prepare(handler_id)
    if not ret_imm:
        handler()
    os._exit(0)
