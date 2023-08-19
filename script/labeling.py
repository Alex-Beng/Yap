# 4.0 进度 6436

import os
import cv2
import numpy as np
from PIL import Image
from pytesseract import image_to_string
import json
import argparse

def js_dp(obj, path):
    json.dump(obj, open(path, 'w', encoding='utf-8'), ensure_ascii=False)

def js_ld(path):
    return json.load(open(path, 'r', encoding='utf-8'))

def exist_or_create_json(path):
    if not os.path.exists(path):
        js_dp([], path)


parser = argparse.ArgumentParser()
parser.add_argument('-f', type=str, required=True, help="需要标注的文件夹", default='')
parser.add_argument('--start', type=int, required=False, help="开始的idx", default=0)
parser.add_argument('--end', type=int, required=False, help="结束的idx", default=0)

args = parser.parse_args()
print(args.f, args.start, args.end)


root_path = args.f
x_path = os.path.join(root_path, 'x.json')
y_path = os.path.join(root_path, 'y.json')
exist_or_create_json(x_path)
exist_or_create_json(y_path)
x = js_ld(x_path)
y = js_ld(y_path)

assert(len(x) == len(y))


print('labeled:', len(y))
print(len(set(y)))

idx = args.start

nn2name = {}
nn2label = {}
for file_name in os.listdir(root_path):

    if file_name.endswith('_raw.jpg'):
        t_wds = file_name.split('_')
        # 因为多区域策略，一个idx有多张图
        # print(f'nn: {int(t_wds[0])} {int(t_wds[1])}')
        nn = int(t_wds[0])*10 + int(t_wds[1])
        lb = t_wds[2]
        nn2name[nn] = file_name
        nn2label[nn] = lb

nns = sorted(list(nn2name.keys()))
nns2idx = {} # 用于回退上一张图片

for i in range(len(nns)):
    nns2idx[nns[i]] = i
# print(nn2name, nn2label, nns2idx)


print('total:', len(nns))
# print(list(idx2label.items())[:10])
# 记录时间
import time
beg_idx = idx
start = time.time()

print(f'cccc{idx}, {len(nns)}')
while idx <= args.end and idx <= len(nns):
    nn = nns[idx]
    path = os.path.join(root_path, nn2name[nn])
    print(f'labeling, {idx}, {path}')
    img = Image.open(path)
    # img = cv2.imread(path)
    img = cv2.cvtColor(np.array(img), cv2.COLOR_RGB2BGR)
    # img = cv2.resize(img, (145, 32))
    img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)

    if (beg_idx - idx - 1)%100 == 0:
        print()
        print()
        print()
        print(f'tpm = {(idx - beg_idx) / (time.time() - start)}')
        print()

    cv2.imshow("raw", img)

    img = cv2.threshold(img, 0, 255, cv2.THRESH_OTSU)[1]
    # img = cv2.copyMakeBorder(img, 0,0,0,384-145, cv2.BORDER_CONSTANT, value=0)
    text2 = image_to_string(img, lang='chi_sim')
    text2 = text2.strip()
    text = nn2label[nn]
    print(f"=a;={text}=={text==text2}\n=z/={text2}==")
    
    cv2.imshow("le", img)
    # print(img.shape)
    cv2.setWindowProperty('le', cv2.WND_PROP_FULLSCREEN, cv2.WINDOW_FULLSCREEN)


    k = cv2.waitKey(0)
    # 我测这个键位左手太累了，本来打字就很多左手了
    # 左右手镜像键位
    # O/W 删除上一个
    # Q 保存退出
    # ;/A 添加第一个作为标签
    # Z// 添加第二个作为标签
    # L/S 添加输入的标签
    # K/D 设为空标签
    # J/F 添加上一个一样的标签
    if k == ord('o') or k == ord('w'):
        x = x[:-1]
        y = y[:-1]
        print(y[-10:])
        print(len(set(y)))
        print(x[-10:])
        idx = nns2idx[
            int(x[-1].split('\\')[-1].split('_')[0])*10 + 
            int(x[-1].split('\\')[-1].split('_')[1])
            ]
        # cnt = int(x[-1].split('\\')[-1].split('_')[0]) if len(x) else args.start-1
    elif k == ord('q'):
        js_dp(x, x_path)
        js_dp(y, y_path)
        break
    elif k == ord(';') or k == ord('a'):
        x.append(path)
        y.append(text)
        print(y[-10:])
        print(len(set(y)))
    elif k == ord('z') or k == ord('/'):
        x.append(path)
        y.append(text2)
        print(y[-10:])
        print(len(set(y)))
    elif k == ord('l') or k == ord('s'):
        text = input("Input the text: ")
        x.append(path)
        y.append(text)
        print(y[-10:])
        print(len(set(y)))
    elif k == ord('k') or k == ord('d'):
        text = ''
        x.append(path)
        y.append(text)
        print(y[-10:])
        print(len(set(y)))
    elif k == ord('j') or k == ord('f'):
        # 上一个不为空的text
        ti = -1
        while ti >= -len(y) and y[ti] == '':
            ti -= 1
        text = y[ti]
        x.append(path)
        y.append(text)
        print(y[-10:])
        print(len(set(y)))
    idx += 1

js_dp(x, x_path)
js_dp(y, y_path)

'''
16:24 3498
16:34 4001
d = 503
tps = 503 / 10 = 50.3

16:42 4076
16:52 4723
16:55 4846

16:57 4846
17:07 6100
d = 1254
tps = 1254 / 10 = 125.4

17:27 7177
d = 1077
tps = 1077 / 10 = 107.7


'''