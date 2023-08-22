import os
import json
import cv2
from PIL import Image
import numpy as np

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
]

error_paths = set([
    # "dumps/17_raw.jpg",
    # "dumps/1773_raw.jpg",
    # "dumps/2878_raw.jpg",
    # "dumps/3079_raw.jpg",
    # "dumps/3421_raw.jpg",
    # "dumps/9279_raw.jpg",
    # "dumps/13062_raw.jpg",
    # "dumps/14761_raw.jpg",
    # "text_dumps/12_raw.jpg",
    # "text_dumps/13_raw.jpg",
    # "text_dumps/14_raw.jpg",
    # "text_dumps/15_raw.jpg",
    # "text_dumps/25_raw.jpg",
    # "text_dumps/26_raw.jpg",
    # "text_dumps/28_raw.jpg",
    # "text_dumps/29_raw.jpg",
    # "text_dumps/5496_raw.jpg",
    # "text_dumps/5497_raw.jpg",
    # "text_dumps/5498_raw.jpg",
    # "text_dumps/8798_raw.jpg",
    # "text_dumps/8799_raw.jpg",
    # "text_dumps/11261_raw.jpg",
    # "text_dumps/15279_raw.jpg",
    # "text_dumps/23792_raw.jpg",
    # "text_dumps/23794_raw.jpg",
    # "text_dumps/32526_raw.jpg",
    # "dumps3/363_混沌容器_raw.jpg",
    # "dumps3/1198_兽肉_raw.jpg",
    # "dumps3/1379_薄荷_raw.jpg",
    # "dumps3/1661_簇_raw.jpg",
    # "dumps3/1868_教官的怀表_raw.jpg",
    # "dumps4.0/733_3_的_raw.jpg",
    # "dumps4.0/1022_2_浊水的一_raw.jpg",
    # "dumps4.0_tx/542_2_异海凝珠_raw.jpg",
    # "dumps4.0_tx/725_2_游医的怀钟_raw.jpg",
    # "dumps4.0_tx/1001_2_调查_raw.jpg",
    # "dumps4.0_tx/1516_4_的时_raw.jpg",
    
    # "dumps/14777_raw.jpg",
    # "dumps/14789_raw.jpg",
    # "dumps/14810_raw.jpg",
    # "dumps/15004_raw.jpg",
    # "dumps/15163_raw.jpg",
    # "dumps/16059_raw.jpg",
    # "text_dumps/8_raw.jpg",
    # "text_dumps/9_raw.jpg",
    # "text_dumps/10_raw.jpg",
    # "text_dumps/11_raw.jpg",
    # "text_dumps/27_raw.jpg",
    # "text_dumps/4216_raw.jpg",
    # "text_dumps/4401_raw.jpg",
    # "text_dumps/6356_raw.jpg",
    # "text_dumps/23725_raw.jpg",
    # "dumps4.0/1002_2_浊水的一_raw.jpg",
    # "dumps4.0/1007_2_浊水的一_raw.jpg",
    # "dumps4.0/1022_2_浊水的一_raw.jpg",
    # "dumps4.0/1027_2_浊水的一_raw.jpg",
    # "dumps4.0/1268_3_浊水的一_raw.jpg",
    # "dumps4.0_tx/88_2_浊水的一_raw.jpg",
    # "dumps4.0_tx/1943_3_浊水的一_raw.jpg",
    
    # "dumps/563_raw.jpg",
    # "dumps/3713_raw.jpg",
    # "dumps/6703_raw.jpg",
    # "dumps/12515_raw.jpg",
    # "dumps/14750_raw.jpg",
    # "dumps/14760_raw.jpg",
    # "dumps/14761_raw.jpg",
    # "dumps/14777_raw.jpg",
    # "dumps/14789_raw.jpg",
    # "dumps/14808_raw.jpg",
    # "dumps/14810_raw.jpg",
    # "dumps/15135_raw.jpg",
    # "dumps/15192_raw.jpg",
    # "text_dumps/6_raw.jpg",
    # "text_dumps/7_raw.jpg",
    # "text_dumps/9153_raw.jpg",
    # "text_dumps/25105_raw.jpg",
    # "dumps4.0_tx/18_3_浊水的一_raw.jpg",
    # "dumps4.0_tx/54_3_浊水的_raw.jpg",


    # "dumps/16061_raw.jpg",
    # "text_dumps/9153_raw.jpg",
    # "dumps3/227_战脉的枯叶_raw.jpg",
    # "dumps4.0_tx/175_2_浊水的一_raw.jpg",
    # "dumps4.0_tx/535_2_异海凝_raw.jpg",
    

    # "dumps/15006_raw.jpg",
    # "dumps/15018_raw.jpg",
    # "dumps/15111_raw.jpg",
    # "dumps/15198_raw.jpg",
    # "dumps4.0/1477_2_异海凝_raw.jpg",
    # "dumps4.0_tx/87_2_浊水的一_raw.jpg",
])


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
        # print(x[i])
    

    # yap-train clean_up.py -> error_paths
    for i, pt in enumerate(x):
        # print(pt)
        if pt in error_paths:
            print(pt)
            with Image.open(pt) as img:
                
                img = cv2.cvtColor(np.array(img), cv2.COLOR_RGB2BGR)
                # cv2.imshow('img1', img)
                # cv2.imwrite('cmdd17_raw.jpg', img)
                img = cv2.resize(img, (145, 32))
                img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
                img = cv2.threshold(img, 0, 255, cv2.THRESH_OTSU)[1]
                img = cv2.copyMakeBorder(img, 0,0,0,384-145, cv2.BORDER_CONSTANT, value=255)
                ya = Image.fromarray(img)
                ya.show()

                lb = input(f"{y[i]}|input label: ")

                y[i] = lb
                ya.close()
                

    for i, lb in enumerate(y):
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