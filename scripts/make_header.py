header = bytearray()

length = input("Length: ")
header.extend(int(length).to_bytes(3, byteorder='big'))

frame_type = input("Frame type: ")

if frame_type == "data":
    header.append(0x0)

    flags = 0x0

    end_stream = input("end stream? (t/f) ")
    if end_stream == "t":
        flags |= 0x1

    padded = input("padded? (t/f) ")
    if padded == "t":
        flags |= 0x8

    header.append(flags)

elif frame_type == "headers":
    header.append(0x1)

    flags = 0x0

    end_stream = input("end stream? (t/f) ")
    if end_stream == "t":
        flags |= 0x1

    end_headers = input("end headers? (t/f) ")
    if end_headers == "t":
        flags |= 0x4

    padded = input("padded? (t/f) ")
    if padded == "t":
        flags |= 0x8

    priority = input("priority? (t/f) ")
    if priority == "t":
        flags |= 0x20

    header.append(flags)
else:
    raise "unknown type"

stream_identifier = input("Stream identifier: ")
header.extend(int(stream_identifier).to_bytes(4, byteorder='big'))

print(header)
