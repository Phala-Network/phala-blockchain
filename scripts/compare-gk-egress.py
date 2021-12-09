#!/usr/bin/env python
import argparse


def read_record(f):
    while True:
        line = f.readline()
        if not line:
            return None
        if 'payload_hash=' in line:
            break
    items = dict([k.strip(',').split('=') for k in line.split() if '=' in k])
    if 'seq' not in items:
        items['seq'] = items['sequence']
        del items['sequence']
    if 'to' not in items:
        items['to'] = items['destination']
        del items['destination']
    return items


def find_record(n, f, cache):
    r = cache.pop(n, None)
    if r:
        return r
    while True:
        r = read_record(f)
        if not r:
            return None
        seq = int(r['seq'])
        if seq == n:
            return r
        if seq > n:
            cache[seq] = r
            if len(cache) > 100:
                raise Exception('Hard to find seq %d' % n)


def strip_prefix(s):
    # Strip 0x prefix
    return s[2:] if s.startswith('0x') else s


def compare(file_a, file_b):
    buffer_a = {}
    buffer_b = {}

    for seq in range(100000000):
        ra = find_record(seq, file_a, buffer_a)
        rb = find_record(seq, file_b, buffer_b)
        if ra and rb:
            if ra['to'] == rb['to'] and rb['to'] in {
                'phala/gatekeeper/event',
                'phala/gatekeeper/key',
                }:
                continue
            if strip_prefix(ra['payload_hash']) != strip_prefix(rb['payload_hash']):
                print('Mismatch at seq %d' % seq)
                print('a: %s' % ra)
                print('b: %s' % rb)
                break
        else:
            print(f'All mq messages match upto seq={seq-1}')
            break


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("file_a", help="File A")
    parser.add_argument("file_b", help="File B")
    args = parser.parse_args()

    with open(args.file_a, 'r') as f:
        with open(args.file_b, 'r') as g:
            compare(f, g)
