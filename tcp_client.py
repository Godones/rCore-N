import socket
import time
import sched
import random
import numpy as np
from threading import Thread, Lock

import threading

RUN_TIME_LIMIT = 5000

global_num = 0
lock = Lock()

global_delay = []

local = threading.local()

matrix_size = 10

def get_matrix_string():
    random_numbers = [str(random.randint(0, 99)) for _ in range(matrix_size * matrix_size)]
    random_string = " ".join(random_numbers)
    return random_string

def req(tcp_socket):
    start_time = time.time()
    send_data = get_matrix_string()
    tcp_socket.send(send_data.encode("utf8"))
    timeout = False
    while True:
        try:
            recv_data = tcp_socket.recv(4096)
            # print(recv_data.decode("utf8"))
            if recv_data:
                break
        except socket.error:
            timeout = True
            print("time out!!")
            break
    
    if not timeout:
        end_time = time.time()
        local.delays.append((end_time - start_time) * 1000000)
    # print('recv data:', recv_data.decode("utf8"))
    loop_monitor(tcp_socket)


def loop_monitor(socket):
    if hasattr(local, "start_time") == False:
        local.start_time = int(round(time.time() * 1000))
    if int(round(time.time() * 1000)) >= local.start_time + RUN_TIME_LIMIT:
        send_data = "close connection"
        socket.send(send_data.encode("utf8"))
        print("close connection")
        merge_local_delay(local.delays)
        return
    s = sched.scheduler(time.time, time.sleep)  # 生成调度器
    s.enter(0.1, 1, req, (socket,))
    s.run()

def connect(index):
    local.delays = []
    global global_num
    tcp_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    tcp_socket.settimeout(5)
    time.sleep(0.2 * index)
    server_addr = ("127.0.0.1", 6201)
    tcp_socket.connect(server_addr)

    send_data = "connect ok?"
    tcp_socket.send(send_data.encode("utf8"))
    recv_data = tcp_socket.recv(1024)

    print('recv connect result:', recv_data.decode("utf8"))
    if recv_data.decode("utf8") == "connect ok":
        with lock:
            global_num += 1

    while True:
        with lock:
            if global_num == threads_num:
                break
    print("all threads connect success!")
    loop_monitor(tcp_socket)


def merge_local_delay(local_delay):
    global global_delay
    with lock:
        global_delay = global_delay + local_delay

def statistic():
    global global_delay
    with lock:
        array = np.array(global_delay)
        mean = np.mean(array)
        variance = np.var(array)
        print("throughput: ", array.size)
        print("avg delay: ", mean)
        print("variance: ", variance)
       

threads = []
threads_num = 32
for i in range(threads_num):
    t = Thread(target=connect, args=(i,))
    threads.append(t)
    t.start()

for i in range(threads_num):
    threads[i].join()

statistic()
