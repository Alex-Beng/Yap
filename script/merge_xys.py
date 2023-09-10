import os
import json

def js_dp(obj, path):
    json.dump(obj, open(path, 'w', encoding='utf-8'), ensure_ascii=False)

def js_ld(path):
    return json.load(open(path, 'r', encoding='utf-8'))


def exist_or_create_json(path):
    if not os.path.exists(path):
        js_dp([], path)

from common import js_dp, js_ld, exist_or_create_json, root_paths

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
