#!/usr/bin/env python
import re


REGEX = re.compile(r'([^, =]*)=([^, \n]*)')


def iter_mq_emit(in_file):
    for line in in_file:
        if 'Sending message' in line and 'from=Gatekeeper' in line:
            yield line

def read_msgs(filename):
    return iter_mq_emit(open(filename))


def parse(line):
    return dict(REGEX.findall(line))


def diff(left_file, right_file):
    n = 0
    for l, r in zip(read_msgs(left_file), read_msgs(right_file)):
        n += 1
        info_l = parse(l)
        info_r = parse(r)

        hash_l = info_l.pop('payload_hash')
        hash_r = info_r.pop('payload_hash')

        diver_fileds = ('phala/gatekeeper/event', 'phala/gatekeeper/key')

        if info_l != info_r or (hash_l != hash_r and info_l['to'] not in diver_fileds):
            print("Mq egress mismatch: left: {} right: {}".format(info_l, info_r))
            print("    left hash: {}".format(hash_l))
            print("   right hash: {}".format(hash_r))
            break
    print('{} messages checked'.format(n))


if __name__ == '__main__':
    import sys
    import argparse

    parser = argparse.ArgumentParser()
    parser.add_argument("file_a", help="File A")
    parser.add_argument("file_b", help="File B")

    args = parser.parse_args()

    diff(args.file_a, args.file_b)
