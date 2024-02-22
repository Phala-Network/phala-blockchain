#!/usr/bin/env python

import sys
import codecs

attrs = {}
for line in sys.stdin:
    print(line.rstrip())
    key, value = line.split(':', 1)
    attrs[key.strip()] = value.strip()

def hex16(n):
    n = int(n)
    l = n & 0xff
    h = (n > 8 )& 0xff
    return codecs.encode(bytes([l, h]), 'hex').decode()

measurement = attrs['mr_enclave'] + hex16(attrs['isv_prod_id']) + hex16(attrs['isv_svn']) + attrs['mr_signer']
print("pruntime_finger_print: " + measurement)
