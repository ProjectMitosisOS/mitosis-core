import os
import sys

sys.path.append("../../common")  # include outer path

import bench
from mitosis_wrapper import *
from chameleon import PageTemplate
import six

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'


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


@tick_execution_time
def handler():
    num_of_rows = 4
    num_of_cols = 4

    tmpl = PageTemplate(BIGTABLE_ZPT)
    data = {}
    for i in range(num_of_cols):
        data[str(i)] = i

    table = [data for x in range(num_of_rows)]
    options = {'table': table}
    data = tmpl.render(options=options)

@mitosis_bench
def bench():
    handler()


if __name__ == '__main__':
    bench()