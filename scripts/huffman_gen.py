"""
Copyright 2017 ThetaSinner

This file is part of Osmium.

Osmium is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

Osmium is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with Osmium.  If not, see <http://www.gnu.org/licenses/>.
"""

TABLE = [
    (0x1ff8, 13),
    (0x7fffd8, 23),
    (0xfffffe2, 28),
    (0xfffffe3, 28),
    (0xfffffe4, 28),
    (0xfffffe5, 28),
    (0xfffffe6, 28),
    (0xfffffe7, 28),
    (0xfffffe8, 28),
    (0xffffea, 24),
    (0x3ffffffc, 30),
    (0xfffffe9, 28),
    (0xfffffea, 28),
    (0x3ffffffd, 30),
    (0xfffffeb, 28),
    (0xfffffec, 28),
    (0xfffffed, 28),
    (0xfffffee, 28),
    (0xfffffef, 28),
    (0xffffff0, 28),
    (0xffffff1, 28),
    (0xffffff2, 28),
    (0x3ffffffe, 30),
    (0xffffff3, 28),
    (0xffffff4, 28),
    (0xffffff5, 28),
    (0xffffff6, 28),
    (0xffffff7, 28),
    (0xffffff8, 28),
    (0xffffff9, 28),
    (0xffffffa, 28),
    (0xffffffb, 28),
    (0x14, 6),
    (0x3f8, 10),
    (0x3f9, 10),
    (0xffa, 12),
    (0x1ff9, 13),
    (0x15, 6),
    (0xf8, 8),
    (0x7fa, 11),
    (0x3fa, 10),
    (0x3fb, 10),
    (0xf9, 8),
    (0x7fb, 11),
    (0xfa, 8),
    (0x16, 6),
    (0x17, 6),
    (0x18, 6),
    (0x0, 5),
    (0x1, 5),
    (0x2, 5),
    (0x19, 6),
    (0x1a, 6),
    (0x1b, 6),
    (0x1c, 6),
    (0x1d, 6),
    (0x1e, 6),
    (0x1f, 6),
    (0x5c, 7),
    (0xfb, 8),
    (0x7ffc, 15),
    (0x20, 6),
    (0xffb, 12),
    (0x3fc, 10),
    (0x1ffa, 13),
    (0x21, 6),
    (0x5d, 7),
    (0x5e, 7),
    (0x5f, 7),
    (0x60, 7),
    (0x61, 7),
    (0x62, 7),
    (0x63, 7),
    (0x64, 7),
    (0x65, 7),
    (0x66, 7),
    (0x67, 7),
    (0x68, 7),
    (0x69, 7),
    (0x6a, 7),
    (0x6b, 7),
    (0x6c, 7),
    (0x6d, 7),
    (0x6e, 7),
    (0x6f, 7),
    (0x70, 7),
    (0x71, 7),
    (0x72, 7),
    (0xfc, 8),
    (0x73, 7),
    (0xfd, 8),
    (0x1ffb, 13),
    (0x7fff0, 19),
    (0x1ffc, 13),
    (0x3ffc, 14),
    (0x22, 6),
    (0x7ffd, 15),
    (0x3, 5),
    (0x23, 6),
    (0x4, 5),
    (0x24, 6),
    (0x5, 5),
    (0x25, 6),
    (0x26, 6),
    (0x27, 6),
    (0x6, 5),
    (0x74, 7),
    (0x75, 7),
    (0x28, 6),
    (0x29, 6),
    (0x2a, 6),
    (0x7, 5),
    (0x2b, 6),
    (0x76, 7),
    (0x2c, 6),
    (0x8, 5),
    (0x9, 5),
    (0x2d, 6),
    (0x77, 7),
    (0x78, 7),
    (0x79, 7),
    (0x7a, 7),
    (0x7b, 7),
    (0x7ffe, 15),
    (0x7fc, 11),
    (0x3ffd, 14),
    (0x1ffd, 13),
    (0xffffffc, 28),
    (0xfffe6, 20),
    (0x3fffd2, 22),
    (0xfffe7, 20),
    (0xfffe8, 20),
    (0x3fffd3, 22),
    (0x3fffd4, 22),
    (0x3fffd5, 22),
    (0x7fffd9, 23),
    (0x3fffd6, 22),
    (0x7fffda, 23),
    (0x7fffdb, 23),
    (0x7fffdc, 23),
    (0x7fffdd, 23),
    (0x7fffde, 23),
    (0xffffeb, 24),
    (0x7fffdf, 23),
    (0xffffec, 24),
    (0xffffed, 24),
    (0x3fffd7, 22),
    (0x7fffe0, 23),
    (0xffffee, 24),
    (0x7fffe1, 23),
    (0x7fffe2, 23),
    (0x7fffe3, 23),
    (0x7fffe4, 23),
    (0x1fffdc, 21),
    (0x3fffd8, 22),
    (0x7fffe5, 23),
    (0x3fffd9, 22),
    (0x7fffe6, 23),
    (0x7fffe7, 23),
    (0xffffef, 24),
    (0x3fffda, 22),
    (0x1fffdd, 21),
    (0xfffe9, 20),
    (0x3fffdb, 22),
    (0x3fffdc, 22),
    (0x7fffe8, 23),
    (0x7fffe9, 23),
    (0x1fffde, 21),
    (0x7fffea, 23),
    (0x3fffdd, 22),
    (0x3fffde, 22),
    (0xfffff0, 24),
    (0x1fffdf, 21),
    (0x3fffdf, 22),
    (0x7fffeb, 23),
    (0x7fffec, 23),
    (0x1fffe0, 21),
    (0x1fffe1, 21),
    (0x3fffe0, 22),
    (0x1fffe2, 21),
    (0x7fffed, 23),
    (0x3fffe1, 22),
    (0x7fffee, 23),
    (0x7fffef, 23),
    (0xfffea, 20),
    (0x3fffe2, 22),
    (0x3fffe3, 22),
    (0x3fffe4, 22),
    (0x7ffff0, 23),
    (0x3fffe5, 22),
    (0x3fffe6, 22),
    (0x7ffff1, 23),
    (0x3ffffe0, 26),
    (0x3ffffe1, 26),
    (0xfffeb, 20),
    (0x7fff1, 19),
    (0x3fffe7, 22),
    (0x7ffff2, 23),
    (0x3fffe8, 22),
    (0x1ffffec, 25),
    (0x3ffffe2, 26),
    (0x3ffffe3, 26),
    (0x3ffffe4, 26),
    (0x7ffffde, 27),
    (0x7ffffdf, 27),
    (0x3ffffe5, 26),
    (0xfffff1, 24),
    (0x1ffffed, 25),
    (0x7fff2, 19),
    (0x1fffe3, 21),
    (0x3ffffe6, 26),
    (0x7ffffe0, 27),
    (0x7ffffe1, 27),
    (0x3ffffe7, 26),
    (0x7ffffe2, 27),
    (0xfffff2, 24),
    (0x1fffe4, 21),
    (0x1fffe5, 21),
    (0x3ffffe8, 26),
    (0x3ffffe9, 26),
    (0xffffffd, 28),
    (0x7ffffe3, 27),
    (0x7ffffe4, 27),
    (0x7ffffe5, 27),
    (0xfffec, 20),
    (0xfffff3, 24),
    (0xfffed, 20),
    (0x1fffe6, 21),
    (0x3fffe9, 22),
    (0x1fffe7, 21),
    (0x1fffe8, 21),
    (0x7ffff3, 23),
    (0x3fffea, 22),
    (0x3fffeb, 22),
    (0x1ffffee, 25),
    (0x1ffffef, 25),
    (0xfffff4, 24),
    (0xfffff5, 24),
    (0x3ffffea, 26),
    (0x7ffff4, 23),
    (0x3ffffeb, 26),
    (0x7ffffe6, 27),
    (0x3ffffec, 26),
    (0x3ffffed, 26),
    (0x7ffffe7, 27),
    (0x7ffffe8, 27),
    (0x7ffffe9, 27),
    (0x7ffffea, 27),
    (0x7ffffeb, 27),
    (0xffffffe, 28),
    (0x7ffffec, 27),
    (0x7ffffed, 27),
    (0x7ffffee, 27),
    (0x7ffffef, 27),
    (0x7fffff0, 27),
    (0x3ffffee, 26),
    (0x3fffffff, 30)
]

class TreeNode:
    def __init__(self, val):
        self.val = val
        self.left = None
        self.right = None


def make_huffman_tree():
    root = TreeNode(None)

    index = 0
    for row in TABLE:
        working_node = root

        for bit_number in range(row[1] - 1, -1, -1):
            if (1 << bit_number) & row[0] == 1 << bit_number:
                if working_node.right == None:
                    working_node.right = TreeNode(None)

                working_node = working_node.right
            else:
                if working_node.left == None:
                    working_node.left = TreeNode(None)

                working_node = working_node.left
        
        working_node.val = index
        index += 1

    return root


def decode(huffman_string, huffman_tree):
    string = ""

    working_node = huffman_tree
    last_bit_number = 0

    for octet in huffman_string:
        for bit_number in range(7, -1, -1):
            if octet & (1 << bit_number) == 1 << bit_number:
                working_node = working_node.right
            else:
                working_node = working_node.left

            if working_node.val != None:
                string = string + chr(working_node.val)
                # go back to the root node
                working_node = huffman_tree
                # track how many bits of the current octet have been used
                last_bit_number = bit_number

    # note that the bit_number in the loop above counts down
    if last_bit_number != 0:
        if last_bit_number > 7:
            raise "Huffman decoding error, too many EOS bits."

        if huffman_string[len(huffman_string) - 1] & ((1 << last_bit_number) - 1) != ((1 << last_bit_number) - 1):
            raise "Huffman decoding error, encoded string does not end with EOS."

    # return the decoded string and the number of bits not used
    return string

# The number of fast lookup tables is 1 less than the number of symbols in the given alphabet (TABLE).
# This is the same as the number of interior nodes in the huffman tree plus the root node.
def make_fast_tables(huffman_table, huffman_tree):
    tables = []

    null_table = _make_fast_table(huffman_table, huffman_tree, (0, 0))
    tables.append(null_table)

    remainders = set(map(lambda row: row[2], null_table))
    finished_remainders = set((0, 0))

    while remainders - finished_remainders:
        remainder = (remainders - finished_remainders).pop()

        table = _make_fast_table(huffman_table, huffman_tree, remainder)
        tables.append(table)

        # add any new remainders to the remainders set
        remainders |= set(map(lambda row: row[2], table))

        # add the remainder which was just processed to the finished set
        finished_remainders.add(remainder)

    return tables


# remainder is a 2-tuple with a remainder value from a previous table and the number of bits in the value.
def _make_fast_table(table, huffman_tree, remainder):
    output = []

    working_root_node = huffman_tree

    # move to the node in the tree indicated by the remainder
    for bit_number in range(remainder[1] - 1, -1, -1):
        if remainder[0] & (1 << bit_number) == 1 << bit_number:
            working_root_node = working_root_node.right
        else:
            working_root_node = working_root_node.left

        # If the bit pattern in the remainder can be decoded to a symbol then it should have happened
        # in a previous table, so this is an error.
        if working_root_node.val != None:
            print(remainder)
            raise Exception("Unexpected value while processing remainder for fast table")

    for i in range(0, 256):
        emit = ""

        working_node = working_root_node
        last_bit_number = 8
        
        for bit_number in range(7, -1, -1):
            if i & (1 << bit_number) == 1 << bit_number:
                working_node = working_node.right
            else:
                working_node = working_node.left

            if working_node.val != None:
                emit += chr(working_node.val)
                working_node = huffman_tree
                last_bit_number = bit_number

        key = (remainder[0] << 8) + i
        rem_value = i & ((1 << last_bit_number) - 1)

        if not emit:
            # we haven't been able to find any matches on the remainder followed by this bit pattern
            # so we forward the remainder with this bit pattern as a new remainder
            rem_value += remainder[0] << 8
            last_bit_number += remainder[1]
        
        output.append((key, emit, (rem_value, last_bit_number)))

    return output


def fast_decode():
    pass
    

def print_fast_table(table):
    for row in table:
        print("{:<12b} | {:<3} | {},{}".format(row[0], row[1], row[2][0], row[2][1]))


huffman_tree = make_huffman_tree()

if huffman_tree.right.left.right.right.right.left.right.val == 66:
    print("Tree looks okay")
else:
    print("Tree is not okay")

if "Hello, world!" == decode([198, 90, 40, 63, 210, 158, 15, 101, 18, 127, 31], huffman_tree):
    print("Decoder looks okay")
else:
    print("Decoder is not okay")

fast_tables = make_fast_tables(TABLE, huffman_tree)
#assert 256 == len(fast_tables)

print_fast_table(fast_tables[1])

#fast_tables = list(map(lambda row: list(map(lambda _row: (_row[0], _row[1], _row[2][0]), row)), fast_tables))
#assert 256 == len(fast_tables)
