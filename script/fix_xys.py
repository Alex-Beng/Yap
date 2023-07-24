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
    'dumps3'
]

xx = []
yy = []

for rp in root_paths:
    x_path = os.path.join(rp, 'x.json')
    y_path = os.path.join(rp, 'y.json')

    x = js_ld(x_path)
    y = js_ld(y_path)
    assert(len(x) == len(y))
    for i, pt in enumerate(x):
        x[i] = pt.replace("\\", "/")
        if pt[0] == '/':
            x[i] = pt[1:]

    for i, lb in enumerate(y):
        # 修复错误label
        if "不详" in lb:
            lb = lb.replace("不详", "不祥")
            y[i] = lb
        if lb == "游区的药壶":
            y[i] = "游医的药壶"
        if lb == " 甜甜花":
            y[i] = "甜甜花"
    js_dp(y, y_path)
    js_dp(x, x_path)