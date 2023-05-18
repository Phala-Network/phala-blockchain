import re
import json


def extract_block_total(line):
    m = re.search(r'block=(\d+), total_paid=(\d+\.\d+), total_treasury=(\d+\.\d+), total=(\d+\.\d+)', line)
    if m:
        return int(m.group(1)), float(m.group(2)), float(m.group(3)), float(m.group(4))
    else:
        return None, None, None, None


def extract_all():
    records = []
    for line in open('emit.log'):
        block, total_paid, total_treasury, total = extract_block_total(line)
        if block:
            records.append((block, total_paid, total_treasury, total))
    json.dump(records, open('records.json', 'w'))


def analyze(end, duration):
    duration=7200
    records = json.load(open('records.json'))
    total = 0
    for b, p, t, tt in records:
        if end - duration < b < end:
            total += tt
    print('total:', total)

def analyze_all():
    duration=7200
    records = json.load(open('records.json'))
    start = 0
    for b, p, t, tt in records:
        if start == 0:
            start = b
            total = 0
        total += tt
        if b - start > duration:
            print(f'from {start} to {b}, issued {total}')
            start = 0

# analyze_all()
def on_chain_gk_mq():
    def seq(line):
        m = re.search(r'block=(\d+), seq=(\d+)', line)
        if m:
            return int(m.group(2))
        else:
            return None
    out = open('gk-extrinsics-w.log', 'w')
    next_seq = None
    for line in open('gk-extrinsics.log'):
        s = seq(line)
        if s is None:
            continue
        if next_seq is None or s == next_seq:
            out.write(line)
            next_seq = s + 1

def diff():
    fa = open('on-chain-gk-payload.log')
    fb = open('replay-payload.log')
    for i, (a, b) in enumerate(zip(fa, fb)):
        if a != b:
            print(a, b)
            break
        else:
            print(i, 'ok')

def check_seq():
    def seq(line):
        m = re.search(r'block=(\d+), seq=(\d+)', line)
        if m:
            return int(m.group(2))
        else:
            return None
    next_seq = None
    for line in open('gk-khala-extrinsics.log'):
        s = seq(line)
        if s is None:
            continue
        if next_seq is None or s == next_seq:
            next_seq = s + 1
        else:
            print(s, next_seq)
    print(next_seq)
    print('ok')


def analyze_settled():
    duration=7200
    start = 0
    total = 0
    pb = 0
    for line in open('scripts/js/settle.log'):
        b, paid = [int(x.strip()) for x in line.split(',')]
        if b > pb:
            print(b, b - pb)
        pb = b
        if b - start > duration:
            # print(f'from {start} to {b}, settled {(total>>48)/0x10000}')
            start = 0
        if start == 0:
            start = b
            total = 0
        total += paid

analyze_settled()
