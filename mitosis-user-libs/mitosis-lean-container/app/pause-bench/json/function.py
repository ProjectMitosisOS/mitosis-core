import json
import os
import socket

server_addr = 'uds.socket'
socket_family = socket.AF_UNIX
socket_type = socket.SOCK_STREAM

try:
    os.remove(server_addr)
except:
    pass

my_socket = socket.socket(socket_family, socket_type)
my_socket.bind(server_addr)
my_socket.listen(1)

connection, client_address = my_socket.accept()

f = open('linux.json')
content = f.read()

def handler():
    json_data = json.loads(content)
    str_json = json.dumps(json_data, indent=4)

while True:
    connection.recv(1)
    handler()
    connection.sendall('\0')
