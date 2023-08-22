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
for i, lb in enumerate(y):
    if lb == "幸运儿银":
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
