import socket

my_socket = socket.socket()
my_socket.connect(("127.0.0.1", 8080))

data = bytes([
    0x0, 0x0, 0x5, # 3 bytes for length
    0x0, # 1 byte for type
    0x0, # 1 byte for flags
    0x0, 0x0, 0x0, 0x6 # 4 bytes for stream id
])
my_socket.send(data)
my_socket.send(bytes([0x0, 0x0, 0x0, 0x0, 0x0]))

response = my_socket.recv(1024)
my_socket.close()
print(response)
