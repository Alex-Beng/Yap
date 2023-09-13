
# y_min = float('inf')
# y_max = float('-inf')

# int2times = dict()
# for r in range(256):
#     for g in range(256):
#         for b in range(256):
#             y = 1.0 * r + 4.5906 * g + 0.0601 * b
#             int2times[int(y)] = int2times.get(int(y), 0) + 1
#             y_min = min(y_min, y)
#             y_max = max(y_max, y)
# print(int2times)
# print(y_min, y_max)

# 绘制折线图
img_path = "text_dumps/27434_raw.jpg"
import cv2

img = cv2.imread(img_path)
img_lab = cv2.cvtColor(img, cv2.COLOR_BGR2LAB)

img_l = img_lab[:, :, 0]
# img_l = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
# print(img_l.shape)
thr, img_l_thr = cv2.threshold(img_l, 0, 255, cv2.THRESH_OTSU)

cv2.imshow("img_l", img_l)
cv2.waitKey(0)

# 统计L通道的直方图
L2times = dict()
for i in range(img_l.shape[0]):
    for j in range(img_l.shape[1]):
        L2times[img_l[i, j]] = L2times.get(img_l[i, j], 0) + 1
# 绘制折线图

import matplotlib.pyplot as plt
x = [i for i in range(256)]
y = [L2times.get(i, 0) for i in range(256)]
# 绘制阈值
plt.plot([thr, thr], [0, max(y)], color='red')
plt.plot(x, y)
plt.xlabel("L* channel/value")
plt.ylabel("times")
plt.show()


