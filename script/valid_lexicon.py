import json
from PIL import Image
import cv2
import numpy as np

all_name = json.load(open('./models/all_list.json', 'r', encoding='utf-8'))

lexicon = set({})
for name in all_name:
    for char in name:
        lexicon.add(char)

lexicon = sorted(list(lexicon))

x = json.load(open('./xx.json', 'r', encoding='utf-8'))
y = json.load(open('./yy.json', 'r', encoding='utf-8'))

not_in_lexicon = set({})
not_in_all_name = set({})
lb2nums = dict()
syfs = set("地脉的枯叶旧枝新芽")
for i, lb in enumerate(y):
    lb2nums[lb] = lb2nums.get(lb, 0) + 1
    # if "战狂的xxx" in lb:
    if "何人所珍藏之" == lb:
    # if set(lb).issubset(syfs) and len(lb) > 10000:
        print(lb)
        img = Image.open(x[i])
        img = np.array(img)
        img = cv2.cvtColor(img, cv2.COLOR_RGB2BGR)
        cv2.imshow('img', img)
        cv2.waitKey(0)
    if lb not in all_name:
        print(lb)
        not_in_all_name.add(lb)
    for c in lb:
        if c not in lexicon:
            print(lb)
            # img = Image.open(x[i])
            # img.show()
            
            not_in_lexicon.add(c)
            break
print(not_in_lexicon)
print(not_in_all_name)
for k in sorted(lb2nums.keys(), key=lambda x:lb2nums[x]):
    print(k, lb2nums[k])

# plot lb 2 nums
import matplotlib.pyplot as plt
plt.rcParams['font.family']=['WenQuanYi Zen Hei']
# plt.figure(figsize=(20, 10))
keys = sorted(lb2nums.keys(), key=lambda x:lb2nums[x])[:-1]
values = [lb2nums[k] for k in keys]
# plt.bar(lb2nums.keys(), lb2nums.values())
# plt.xticks(rotation=90)
plt.bar(keys, values)
plt.xticks(rotation=90)
plt.show()
