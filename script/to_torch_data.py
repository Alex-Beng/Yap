# 不可用

import cv2
import pickle
from PIL import Image

import torch
import torch.nn as nn
import torchvision.transforms as transforms


x = pickle.load(open('x.pk', 'rb'))
y = pickle.load(open('y.pk', 'rb'))
assert(len(x) == len(y))
n = len(x)

xx = []
yy = []
root_path = 'dumps/'
for cnt in range(n):
    text = y[cnt]
    if text != '':
        continue

    img = cv2.imread(f"{root_path}{x[cnt]}_raw.jpg")
    img = cv2.resize(img, (145, 32))
    img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
    img = cv2.threshold(img, 0, 255, cv2.THRESH_OTSU)[1]

    # print(img.shape)
    img1 = cv2.copyMakeBorder(img, 0,0,0,384-145, cv2.BORDER_CONSTANT, value=0)
    img2 = cv2.copyMakeBorder(img, 0,0,0,384-145, cv2.BORDER_CONSTANT, value=255)
    img3 = cv2.copyMakeBorder(img, 0,0,0,384-145, cv2.BORDER_DEFAULT, value=0)
    img4 = cv2.copyMakeBorder(img, 0,0,0,384-145, cv2.BORDER_REFLECT, value=0)
    img5 = cv2.copyMakeBorder(img, 0,0,0,384-145, cv2.BORDER_REPLICATE, value=0)

    # cv2.imshow('Padded Image', img5)    
    # print(img.shape)
    cv2.waitKey(1)
    img = Image.fromarray(img)
    # print(f"{img.size}, {text}")
    for i in range(1, 6):
        img = eval(f"img{i}")
        tensor = transforms.ToTensor()(img)
        tensor = torch.unsqueeze(tensor, dim=0)
        xx.append(tensor)
        yy.append(text)
    if cnt%100 == 0:
        print(cnt)
print(len(xx))
xxx = torch.cat(xx, dim=0)
torch.save(xxx, 'zero_x.pt')
torch.save(yy, 'zero_y.pt')