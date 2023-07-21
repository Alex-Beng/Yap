import cv2
from pytesseract import image_to_string
import pickle
import argparse

parser = argparse.ArgumentParser()
parser.add_argument('--start', type=int, default=0)
parser.add_argument('--end', type=int, default=0)

args = parser.parse_args()
print(args.start, args.end)


root_path = 'dumps/'
x = pickle.load(open('x.pk', 'rb'))
y = pickle.load(open('y.pk', 'rb'))

print('labeled:', len(y))
# print(y[-10:])
print(len(set(y)))
cnt = args.start

# 记录时间
import time
beg_cnt = cnt
start = time.time()

while cnt <= args.end:
    

    path = f'{root_path}{cnt}_raw.jpg'
    print(f'labeling, {path}')
    img = cv2.imread(path)
    img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)

    if (beg_cnt - cnt - 1)%100 == 0:
        print()
        print()
        print()
        print(f'tpm = {(cnt - beg_cnt) / (time.time() - start)}')
        print()

    cv2.imshow("raw", img)

    img = cv2.threshold(img, 127, 255, cv2.THRESH_OTSU)[1]
    text = image_to_string(img, lang='chi_sim')
    text = text.strip()
    print(f"=={text}==")
    
    cv2.imshow("le", img)
    cv2.setWindowProperty('le', cv2.WND_PROP_FULLSCREEN, cv2.WINDOW_FULLSCREEN)


    k = cv2.waitKey(0)
    if k == ord('w'):
        x = x[:-1]
        y = y[:-1]
        print(y[-10:])
        print(len(set(y)))
        cnt -= 2
    elif k == ord('q'):
        pickle.dump(x, open('x.pk', 'wb'))
        pickle.dump(y, open('y.pk', 'wb'))
        break
    elif k == ord('a'):
        x.append(cnt)
        y.append(text)
        print(y[-10:])
        print(len(set(y)))
    elif k == ord('s'):
        text = input("Input the text: ")
        x.append(cnt)
        y.append(text)
        print(y[-10:])
        print(len(set(y)))
    elif k == ord('d'):
        text = ''
        x.append(cnt)
        y.append(text)
        print(y[-10:])
        print(len(set(y)))
    elif k == ord('j'):
        text = y[-1]
        x.append(cnt)
        y.append(text)
        print(y[-10:])
        print(len(set(y)))
    elif k == ord('f'):
        # 上一个不为空的text
        ti = -1
        while ti >= -len(y) and y[ti] == '':
            ti -= 1
        text = y[ti]
        x.append(cnt)
        y.append(text)
        print(y[-10:])
        print(len(set(y)))
    cnt += 1

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