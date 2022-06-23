import socket
import sys
import time

master_port = 7000
trigger_port = 9000

FINISH = "finish"
ALL_DONE = "all done"

host = ''
start = time.time()


def report(name, start, end):
    passed_us = (end - start) * 1000000
    print("[%s] duration: %.2f ms" % (str(name), passed_us / 1000))
    sys.stdout.flush()


def server():
    global start, end
    ip_port = (host, master_port)
    server = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)  # udp协议
    server.bind(ip_port)
    process_cnt = 0
    cnt = 0

    while True:
        data, client_addr = server.recvfrom(4096)
        data = data.decode()
        if data == FINISH:
            cnt += 1
        else:
            cnt = 0
            process_cnt = int(data)
            start = time.time()
        if process_cnt <= cnt:
            end = time.time()
            report("rule %d" % cnt, start, end)


server()
