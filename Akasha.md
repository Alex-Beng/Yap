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

# 模板匹配耗时

using SumOfSquaredErrors
```
1 use: 6.5174ms
2 use: 4.8972ms
3 use: 5.6497ms
4 use: 5.5215ms
5 use: 7.1212ms
6 use: 5.9446ms
7 use: 7.0301ms
```
avg: 6.004ms

using SumOfSquaredErrorsNormalized
```
1 use: 7.1286ms
2 use: 6.7157ms
3 use: 6.8852ms
4 use: 7.3145ms
5 use: 6.4137ms
6 use: 6.3717ms
7 use: 5.7694ms
```
avg: 6.676ms

using CrossCorrelation
```
1 use: 5.4954ms
2 use: 5.2545ms
3 use: 5.1441ms
4 use: 5.644ms
5 use: 6.0413ms
6 use: 4.4601ms
7 use: 6.5377ms
```
avg: 5.548ms

using CrossCorrelationNormalized
```
1 use: 7.0845ms
2 use: 5.7376ms
3 use: 6.151ms
4 use: 6.003ms
5 use: 6.8834ms
6 use: 4.5365ms
7 use: 7.1115ms
```
avg: 6.188ms