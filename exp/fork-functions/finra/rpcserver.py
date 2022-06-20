import json
import socket


class TCPServer(object):
    def __init__(self):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    def bind_listen(self, port):
        self.sock.bind(('0.0.0.0', port))
        self.sock.listen(5)

    def accept_receive_close(self):
        (client_socket, address) = self.sock.accept()
        msg = client_socket.recv(4096)
        data = self.on_msg(msg)
        client_socket.sendall(data)
        client_socket.close()


class JSONRPC(object):
    def __init__(self):
        self.data = None

    def from_data(self, data):
        self.data = json.loads(data.decode('utf-8'))

    def call_method(self, data):
        self.from_data(data)
        method_name = self.data['method_name']
        method_args = self.data['method_args']
        method_kwargs = self.data['method_kwargs']
        res = self.funs[method_name](*method_args, **method_kwargs)
        data = {"res": res}
        return json.dumps(data).encode('utf-8')


class RPCStub(object):
    def __init__(self):
        self.funs = {}

    def register_function(self, function, name=None):
        if name is None:
            name = function.__name__
        self.funs[name] = function


class RPCServer(TCPServer, JSONRPC, RPCStub):
    """
    RPC server written in Python.

        s = RPCServer()
        s.register_function(add)
        s.loop(8080)
    """
    def __init__(self):
        TCPServer.__init__(self)
        JSONRPC.__init__(self)
        RPCStub.__init__(self)

    def loop(self, port):
        self.bind_listen(port)
        print('Server listen on %d' % port)
        while True:
            self.accept_receive_close()

    def on_msg(self, data):
        return self.call_method(data)


def add(a, b, c=10):
    sum = a + b + c
    print('add called')
    return sum
