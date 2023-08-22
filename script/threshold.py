# 测试otsu与adaptiveThreshold的效果

import os
import cv2
from PIL import Image
import numpy as np

root_path = 'dumps4.0_tx/'

for file_name in os.listdir(root_path):
    if file_name.endswith('_raw.jpg'):
        raw_path = os.path.join(root_path, file_name)
        bin_path = raw_path.replace('_raw.jpg', '_bin.jpg')

        # print(path)
        img = Image.open(raw_path)
        bin_img = Image.open(bin_path).convert('L')
        bin_img = np.array(bin_img)
        
        img = cv2.cvtColor(np.array(img), cv2.COLOR_RGB2BGR)
        # bin_img = cv2.cvtColor(np.array(bin_img), cv2.COLOR_RGB2BGR)

        p_g_img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
        p_bin_img = cv2.adaptiveThreshold(p_g_img, 255, cv2.ADAPTIVE_THRESH_MEAN_C, cv2.THRESH_BINARY, 111, 1)
        p_otsu_img = cv2.threshold(p_g_img, 0, 255, cv2.THRESH_OTSU)[1]
        
        # 将原先bin image 与处理过的bin imgs组合为一张图
        # print(bin_img.shape, p_bin_img.shape, p_otsu_img.shape)
        big_bin_img = np.concatenate((bin_img, p_otsu_img, p_bin_img), axis=0)
        cv2.imshow("raw", img)
        cv2.imshow("bin", big_bin_img)
        k = cv2.waitKey(0)
        if k == ord('q'):
            break