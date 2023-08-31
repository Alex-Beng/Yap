import os
import json

def js_dp(obj, path):
    json.dump(obj, open(path, 'w', encoding='utf-8'), ensure_ascii=False)

def js_ld(path):
    return json.load(open(path, 'r', encoding='utf-8'))


def exist_or_create_json(path):
    if not os.path.exists(path):
        js_dp([], path)

root_paths = [
    'dumps/',
    'text_dumps/',
    'dumps3',
    'dumps4.0',
    'dumps4.0_tx',
    'dumps4.0_tx2',
    'dumps4.0_tx3',
    'dumps4.0_tx4',
    'dumps4.0_tx5',
    'dumps4.0_tx6',
    'dumps4.0_tx7',
    'dumps4.0_pph',
    'dumps4.0_syfs'
]

xx = []
yy = []

for rp in root_paths:
    x_path = os.path.join(rp, 'x.json')
    y_path = os.path.join(rp, 'y.json')

    x = js_ld(x_path)
    y = js_ld(y_path)
    assert(len(x) == len(y))

    xx += x
    yy += y

print(xx[:3])
print(yy[:3])
print(xx[-3:])
print(yy[-3:])
assert(len(xx) == len(yy))
zero_cnt = 0
for y in yy:
    if y == "":
        zero_cnt += 1

print(len(xx), zero_cnt)
js_dp(xx, 'xx.json')
js_dp(yy, 'yy.json')
