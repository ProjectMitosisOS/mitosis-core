import os
import socket

server_addr = 'uds.socket'
socket_family = socket.AF_UNIX
socket_type = socket.SOCK_STREAM

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'
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

try:
    os.remove(server_addr)
except:
    pass

my_socket = socket.socket(socket_family, socket_type)
my_socket.bind(server_addr)
my_socket.listen(1)

connection, client_address = my_socket.accept()

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

while True:
    connection.recv(1)
    handler()
    connection.sendall('\0')
