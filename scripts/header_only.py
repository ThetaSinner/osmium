import array
import socket

frame = bytearray(b'\x00\x00\x1a\x01\x05\x00\x00\x00\x01')

payload = [64, 10, 99, 117, 115, 116, 111, 109, 45, 107, 101, 121, 13, 99, 117, 115, 116, 111, 109, 45, 104, 101, 97, 100, 101, 114]
for i in payload:
    frame.extend(i.to_bytes(1, byteorder='big'))

print(frame)

my_socket = socket.socket()
my_socket.connect(("127.0.0.1", 8080))
my_socket.send(frame)
