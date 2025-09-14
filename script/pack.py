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

# from IPython import embed
# embed()

assert(len(xx) == len(yy))

all_folder = set()

N = len(xx)
for i in range(N):
    path = xx[i]
    folder = os.path.dirname(path)
    all_folder.add(folder)
print(all_folder)
all_folder = list(all_folder)

# zip file
files_to_add = ['xx.json', 'yy.json', 'usage.py']
import zipfile
# 创建一个ZIP文件
with zipfile.ZipFile('YapDataset.zip', 'w', zipfile.ZIP_DEFLATED) as zipf:
    # 先添加xx.json和yy.json
    for file in files_to_add:
        # 检查文件是否存在
        if os.path.exists(file):
            # 将文件添加到ZIP文件中
            zipf.write(file, file)
        else:
            print(f"miss {file}")

    zip_folder = os.getcwd()
    # all_folder = ['dumps4.2/7', 'dumps4.0_pph']
    for folder in all_folder:
        print(f"in folder {folder}")
        # 检查文件夹是否存在
        if os.path.exists(folder):
            # 遍历文件夹中的所有文件和子文件夹
            for root, dirs, files in os.walk(folder):
                fileNum = len(files)
                for i, file in enumerate(files):
                    if i % 100 == 0:
                        print(f'\r {i}/{fileNum}', end="")
                    # 获取文件的完整路径
                    file_path = os.path.join(root, file)
                    # 将文件添加到ZIP文件中
                    zipf.write(file_path, os.path.relpath(file_path, zip_folder))
                print()

