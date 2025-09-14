import os
import json
import cv2
import numpy as np
from PIL import Image



def js_dp(obj, path):
    json.dump(obj, open(path, 'w', encoding='utf-8'), ensure_ascii=False)

def js_ld(path):
    return json.load(open(path, 'r', encoding='utf-8'))

def exist_or_create_json(path):
    if not os.path.exists(path):
        js_dp([], path)


xx = js_ld('xx.json')
yy = js_ld('yy.json')


assert(len(xx) == len(yy))


N = len(xx)
for i in range(N):
    path = xx[i]
    label = yy[i]
    img = Image.open(path) # cv cannot due with Chinese
    img = cv2.cvtColor(np.array(img), cv2.COLOR_RGB2BGR)
    cv2.imshow("raw", img)
    print(f"{i} -> {label}")
    k = cv2.waitKey(0)
    if k == ord('q'):
        break