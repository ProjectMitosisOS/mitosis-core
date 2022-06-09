import os
import sys

sys.path.append("../../common")  # include outer path
from func_bench_wrapper import *

import six
from chameleon import PageTemplate

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


def handler():
    """
    "params": [
                "{\"num_of_rows\":\"200\", \"num_of_cols\":\"200\"}"
            ]
    """
    num_of_rows = 200
    num_of_cols = 200

    tmpl = PageTemplate(BIGTABLE_ZPT)
    data = {}
    for i in range(num_of_cols):
        data[str(i)] = i

    table = [data for x in range(num_of_rows)]
    options = {'table': table}
    data = tmpl.render(options=options)


@func_exec_bench
def bench():
    handler()


if __name__ == '__main__':
    bench()
