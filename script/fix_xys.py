# fix 标签的路径 及 错误标签

import os
import json
import cv2
from PIL import Image
import numpy as np

from common import js_dp, js_ld, exist_or_create_json, root_paths, error_paths, drop_paths

for rp in root_paths:
    x_path = os.path.join(rp, 'x.json')
    y_path = os.path.join(rp, 'y.json')

    x = js_ld(x_path)
    y = js_ld(y_path)
    assert(len(x) == len(y))
    for i, pt in enumerate(x):
        x[i] = pt.replace("\\", "/").replace("\\\\", "/")
        if x[i][0] == '/':
            x[i] = x[i][1:]
        if x[i][:2] == './':
            x[i] = x[i][2:]
        
        # if '.' in x[i]:
        #     print(x[i])
    
    syfs = set("地脉的枯叶旧枝新芽")
    # yap-train clean_up.py -> error_paths
    for i, pt in enumerate(x):
        # print(pt)
        if pt in error_paths:
        # 为子集
        # if set(y[i]).issubset(syfs) and len(y[i]) > 1:
        # if "地脉的xx" == y[i]:
            # print(pt)
            with Image.open(pt) as img:
                
                img = cv2.cvtColor(np.array(img), cv2.COLOR_RGB2BGR)
                # cv2.imshow('img1', img)
                # cv2.imwrite('cmdd17_raw.jpg', img)
                # img = cv2.resize(img, (145, 32))
                img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
                img = cv2.threshold(img, 0, 255, cv2.THRESH_OTSU)[1]
                r, c = img.shape[:2]
                new_c = int(c/r*32 + 0.5)
                img = cv2.resize(img, (new_c, 32))

                img = cv2.copyMakeBorder(img, 0,0,0,384-145, cv2.BORDER_CONSTANT, value=255)
                ya = Image.fromarray(img)
                ya.show()

                lb = input(f"{y[i]}|input label: ")

                y[i] = lb
                ya.close()
    drop = [0]*len(x)
    for i, pt in enumerate(x):
        if pt in drop_paths:
            print(pt)
            drop[i] = 1
    x = [x[i] for i in range(len(x)) if drop[i] == 0]
    y = [y[i] for i in range(len(y)) if drop[i] == 0]

    for i, lb in enumerate(y):
        if "芙" in lb:
            print(x[i], '->', lb)
        if "政击力" == lb:
            # img = Image.open(x[i].replace('_raw.jpg', '_bin.jpg'))
            # img.show()
            # # 关闭窗口
            # img.close()
            lb = "攻击力"
            y[i] = lb
            pass
        if "异海凝粉" == lb:
            lb = "异海凝珠"
            y[i] = lb
            pass

        # 修复错误label
        if "出生" in lb:
            lb = lb.replace("出生", "初生")
            print(lb, x[i])
            y[i] = lb
            
        if "不详" in lb:
            lb = lb.replace("不详", "不祥")
            y[i] = lb
        if lb == "游区的药壶":
            y[i] = "游医的药壶"
        if lb == " 甜甜花":
            y[i] = "甜甜花"
    js_dp(y, y_path)
    js_dp(x, x_path)