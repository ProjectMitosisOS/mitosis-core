import os
import socket

server_addr = 'uds.socket'
socket_family = socket.AF_UNIX
socket_type = socket.SOCK_STREAM

import shutil
import gzip

try:
    os.remove(server_addr)
except:
    pass

my_socket = socket.socket(socket_family, socket_type)
my_socket.bind(server_addr)
my_socket.listen(1)

connection, client_address = my_socket.accept()

def handler():
    dst = 'result'
    src = 'compression'
    shutil.make_archive(dst, 'zip', src)

while True:
    connection.recv(1)
    handler()
    connection.sendall('\0')
