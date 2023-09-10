import os
import json

from common import js_dp, js_ld,  root_paths

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
