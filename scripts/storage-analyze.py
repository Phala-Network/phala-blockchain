import json

from collections import defaultdict

result = json.load(open('pairs.json', 'r'))['result']

stats = defaultdict(int)

prefixes = {
    "Collections": "0x5bef2c5471aa9e955551dc810f5abb399200647b8c99af7b8b52752114831bdb",
    "Nfts": "0x5bef2c5471aa9e955551dc810f5abb39e8d49389c2e23e152fdd6364daadd2cc",
    "Priorities": "0x5bef2c5471aa9e955551dc810f5abb397f6749268d89e15586d82478e7290431",
    "Children": "0x5bef2c5471aa9e955551dc810f5abb39261f5a952a31d4199096219bbfd87740",
    "Resources": "0x5bef2c5471aa9e955551dc810f5abb392111e0df19de9563b58301e5f7e00743",
    "EquippableBases": "0x5bef2c5471aa9e955551dc810f5abb39ef660df27389f71e45f1741b554773fb",
    "EquippableSlots": "0x5bef2c5471aa9e955551dc810f5abb393e26973064c5f9a17e8bfaa18aee3013",
    "Properties": "0x5bef2c5471aa9e955551dc810f5abb39a436740684271e6e2985d7bb452fdf99",
    "Lock": "0x5bef2c5471aa9e955551dc810f5abb39fb1ef94455cc6b5a3840206754686d98",
    "DummyStorage": "0x5bef2c5471aa9e955551dc810f5abb399439307cc9be85229487820a36657c35",
}
klen = len("0x5bef2c5471aa9e955551dc810f5abb399439307cc9be85229487820a36657c35")

lookup = {v: k for k, v in prefixes.items()}

# print(len(result))
for k, v in result:
    sz = len(v) / 2 + len(k) / 2
    store = lookup.get(k[:klen])
    if store:
        stats[store] += sz
    # stats[k[:16]] += sz

for k, v in sorted(stats.items(), key=lambda i: i[1])[::-1]:
    print(f"RmrkCore::{k}", int(v))
    # print(f"{k}", int(v))

