# 不可用

import cv2


root_path = 'dumps/'
for cnt in range(34612, 34693):
    path = f'{root_path}{cnt}_raw.jpg'
    img = cv2.imread(path)
    img = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)

    cv2.imshow("raw", img)

    img = cv2.threshold(img, 127, 255, cv2.THRESH_OTSU)[1]
    
    cv2.imshow("le", img)

    cv2.waitKey(0)