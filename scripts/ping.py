import socket

my_socket = socket.socket()
my_socket.connect(("127.0.0.1", 8080))

data = bytes([
    0x0, 0x0, 0x8, # 3 bytes for length
    0x6, # 1 byte for type
    0x0, # 1 byte for flags
    0x0, 0x0, 0x0, 0x0, # 4 bytes for stream id

    # payload
    0x1, 0x3, 0x5, 0x7, 0x2, 0x4, 0x6, 0x8
])
my_socket.send(data)

response = my_socket.recv(1024)
my_socket.close()
print(response)
