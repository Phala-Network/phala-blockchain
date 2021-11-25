#!/usr/bin/env python
import json
import codecs
import argparse

try:
    # Python2
    import urllib2 as request
except ImportError:
    # Python3
    import urllib.request as request


DEFAULT_URL = 'http://localhost:9933'


def rpc(url, method, params):
    # post json data
    data = {
        "jsonrpc": "2.0",
        "id":1,
        "method": method,
        "params": params,
    }
    data = json.dumps(data).encode('utf-8')
    req = request.Request(url, data)
    req.add_header('Content-Type', 'application/json; charset=utf-8')
    resp = request.urlopen(req)
    return json.loads(resp.read().decode('utf-8'))


def get_metadata(url):
    response = rpc(url, 'state_getMetadata', [])
    if 'result' not in response:
        raise Exception(response['error'])
    result = response['result']
    return codecs.decode(result[2:], 'hex')


def main():
    parser = argparse.ArgumentParser(description='Dump metadata from a running substrate node')
    parser.add_argument('--url', default=DEFAULT_URL, help='HTTP RPC address of the substrate node')
    parser.add_argument('output', nargs='?', default='metadata.scale', help='output file')
    args = parser.parse_args()
    data = get_metadata(args.url)
    open(args.output, 'wb').write(data)


if __name__ == '__main__':
    main()
