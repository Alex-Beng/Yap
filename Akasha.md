# Pickup浮现、消失时间

60FPS，万叶冲刺

||帧数|时间/ms|
|-|-|-|
|开始->F及物品图标出现|8|133|
|开始->F对应的物品图标及文字出现|17|283|
|开始->所有物品图标及文字出现|25|416|
|所有物品稳定出现的时间|18~40|300~666|

# CRNN推理时间

AMD Ryzen 7 5800H + tract-onnx

一次圣遗物词条长度图片推理用时`5-7ms`


# 屏幕捕捉频率/延时

C++ with gdi 约`50+Hz` ≈ `20ms` 

ref: [调用Windows API截图要50ms一张图，那么那些录屏软件是如何做到60FPS的速度的呢？](https://www.zhihu.com/question/267207676/answer/320151035)

# 模拟输入频率/延时

Python + SendInput 约 `1000次/230ms` ≈ `0.23ms`

测试代码
```python
import pydirectinput # for mouse
pydirectinput.PAUSE = 0

import cProfile

def main():
    for i in range(1000):
        pydirectinput.move(1, 0)
cProfile.run('main()')
```