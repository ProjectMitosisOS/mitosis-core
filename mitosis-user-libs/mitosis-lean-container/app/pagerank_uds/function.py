import igraph
import os
import sys
import socket

server_addr = 'uds.socket'
socket_family = socket.AF_UNIX
socket_type = socket.SOCK_STREAM

size = 100000

graph = igraph.Graph.Barabasi(size, 10)

def generate(length):
    letters = string.ascii_lowercase + string.digits
    return ''.join(random.choice(letters) for i in range(length))

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
                    "{\"size\":\"100000\"}"

    :return:
    """
    result = graph.pagerank()

while True:
    connection.recv(1)
    handler()
    connection.sendall('\0')
