# client.py
import json
import socket


class TCPClient(object):
    def __init__(self):
        self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    def connect(self, host, port):
        self.sock.connect((host, port))

    def send(self, data):
        self.sock.send(data)

    def recv(self, length):
        return self.sock.recv(length)


class RPCStub(object):
    def __getattr__(self, function):
        def _func(*args, **kwargs):
            d = {'method_name': function, 'method_args': args, 'method_kwargs': kwargs}
            self.send(json.dumps(d).encode('utf-8'))  # Send message
            data = self.recv(4096)
            return data

        setattr(self, function, _func)
        return _func


class RPCClient(TCPClient, RPCStub):
    """
    c = RPCClient()
    c.connect('127.0.0.1', 8080)
    res = c.add(1, 2, c=3)
    print(f'res: [{res}]')
    """
    pass


