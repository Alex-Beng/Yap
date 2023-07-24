# 不可用

import cv2
import pickle
'''
{'', '微光花蜜', '战狂的时计', '史莱姆清', '风晶蝶', '铁块', '号角', '新兵的徽', '水晶块', '薄荷', '地脉的旧枝', '蒲公英籽', '导能绘卷', '的面', '雾虚花粉', '禁咒绘卷', '新兵的徽记', '卷心菜', '烹饪', '黑晶号角', '不祥的面具', '面具', '箭簇', '沉重号角', '牢固的', '流放者之羽', '铭记之谷
', '苹果', '史莱姆原浆', '史莱姆凝液', '隐兽指瓜', '史菜姆凝液', '地脉的枯叶', '松果', '黑铜号角', '鸟蛋', '白萝卜', '污秽的面具', '混沌装置', '甜甜花', '雾虚草囊', '破损的面具', '兽肉', '不详的面具', '战狂的时', '历战的箭簇', '骗骗花蜜', '牢固的箭簇', '锐利的箭簇', '日落果', '冰雾花', '污秽的面', '饪', '树莓', '教官的茶杯', '隐兽指爪', '封魔绘卷', '风车菊', '隐兽指扑', '簇', '小麦', '胡萝卜', '牢固的箭', '魔晶块', '电气水晶', '的时计', '隐兽利爪'}
'''

x = pickle.load(open('x.pk', 'rb'))
y = pickle.load(open('y.pk', 'rb'))

assert(len(x)==len(y))
n = len(y)
print(n)
exit()
def fix_x(n):
    root_path = 'dumps/'
    x = []
    for cnt in range(0, n):
        path = f'{root_path}{cnt}_raw.jpg'
        img = cv2.imread(path)
        img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
        img = cv2.threshold(img, 127, 255, cv2.THRESH_OTSU)[1]
        x.append(cnt)
    pickle.dump(x, open('x.pk', 'wb'))
# fix_x(n)
# exit()



def show():
    print(set(y))
    keys = set(y)
    k2imgs = dict()
    for i in range(n):
        img = cv2.imread(f"dumps/{i}_raw.jpg")
        text = y[i]
        k2imgs.setdefault(text, [])
        k2imgs[text].append(img)

    for k in keys:
        if k == '':
            continue
        print(k, len(k2imgs[k]))
        for img in k2imgs[k]:
            cv2.imshow("le", img)
            cv2.waitKey(0)

show()
exit()
    
for i in range(n):
    img = x[i]
    text = y[i]

    if not text == '的面':
        continue
    # y[i] = '新兵的徽记'
    # pickle.dump(y, open('y.pk', 'wb'))
    # 将img放大四倍
    img = cv2.resize(img, (0, 0), fx=4, fy=4, interpolation=cv2.INTER_NEAREST)
    # cv2.putText(img, text, (0, 50), cv2.FONT_HERSHEY_SIMPLEX, 1, (255, 255, 255), 2)
    print(f'=={text}==')

    cv2.imshow('img', img)
    k = cv2.waitKey(0)
    if k == ord("c"):
        y[i] = '新兵的徽'
        # pickle.dump(y, open('y.pk', 'wb'))
    