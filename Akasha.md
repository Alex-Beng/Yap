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

# L 通道转换用时

设置：
1. naive的方法实现转换u8 RGB到L通道。
2. 1920x1080 截屏
3. image crate的graystyle转灰度作为对比
4. 转换100次，

L通道耗时       31、37、39、37、38。
gray通道耗时    8、9、10、9、9。

# L 通道与 gray 通道性能对比

L 通道
```
[2023-07-28T16:19:47Z INFO  yap::inference::img_process] res_x = 4, res_y = 187, res_val = 0.01646706
[2023-07-28T16:19:47Z INFO  yap::inference::img_process] res_x = 3, res_y = 197, res_val = 0.016988654
[2023-07-28T16:19:47Z INFO  yap::inference::img_process] res_x = 10, res_y = 199, res_val = 0.016959824
[2023-07-28T16:19:47Z INFO  yap::inference::img_process] res_x = 0, res_y = 11, res_val = 0.022629136
[2023-07-28T16:19:48Z INFO  yap::inference::img_process] res_x = 0, res_y = 11, res_val = 0.022104608
[2023-07-28T16:19:48Z INFO  yap::inference::img_process] res_x = 0, res_y = 11, res_val = 0.022065477
[2023-07-28T16:19:48Z INFO  yap::inference::img_process] res_x = 6, res_y = 12, res_val = 0.022033807
[2023-07-28T16:19:48Z INFO  yap::inference::img_process] res_x = 0, res_y = 13, res_val = 0.022695053
[2023-07-28T16:19:48Z INFO  yap::inference::img_process] res_x = 2, res_y = 196, res_val = 0.018283721
[2023-07-28T16:19:48Z INFO  yap::inference::img_process] res_x = 11, res_y = 63, res_val = 0.020668533
[2023-07-28T16:19:48Z INFO  yap::inference::img_process] res_x = 11, res_y = 67, res_val = 0.020769918
[2023-07-28T16:19:58Z INFO  yap::inference::img_process] res_x = 1, res_y = 65, res_val = 0.025464674
[2023-07-28T16:19:58Z INFO  yap::inference::img_process] res_x = 11, res_y = 80, res_val = 0.02425491
[2023-07-28T16:19:58Z INFO  yap::inference::img_process] res_x = 9, res_y = 83, res_val = 0.023930188
[2023-07-28T16:19:58Z INFO  yap::inference::img_process] res_x = 0, res_y = 85, res_val = 0.023805134
[2023-07-28T16:19:58Z INFO  yap::inference::img_process] res_x = 6, res_y = 116, res_val = 0.0061717066
[2023-07-28T16:19:58Z INFO  yap::inference::img_process] res_x = 6, res_y = 63, res_val = 0.0055507054
[2023-07-28T16:19:58Z WARN  yap::pickupper::pickup_scanner] 不在大名单: 新兵的
[2023-07-28T16:19:58Z INFO  yap::inference::img_process] res_x = 6, res_y = 99, res_val = 0.005455584
[2023-07-28T16:19:58Z INFO  yap::pickupper::pickup_scanner] 拾起: 士官的徽记
[2023-07-28T16:19:59Z INFO  yap::inference::img_process] res_x = 6, res_y = 145, res_val = 0.00525982
[2023-07-28T16:19:59Z INFO  yap::pickupper::pickup_scanner] 拾起: 尉官的徽记
[2023-07-28T16:19:59Z INFO  yap::inference::img_process] res_x = 6, res_y = 198, res_val = 0.005237456
[2023-07-28T16:19:59Z INFO  yap::pickupper::pickup_scanner] 拾起: 攫金鸦印
[2023-07-28T16:19:59Z INFO  yap::inference::img_process] res_x = 6, res_y = 228, res_val = 0.00442058
[2023-07-28T16:19:59Z INFO  yap::pickupper::pickup_scanner] 拾起: 藏银鸦印
[2023-07-28T16:19:59Z INFO  yap::inference::img_process] res_x = 6, res_y = 260, res_val = 0.0045337477
[2023-07-28T16:19:59Z INFO  yap::pickupper::pickup_scanner] 拾起: 寻宝鸦印
[2023-07-28T16:19:59Z INFO  yap::inference::img_process] res_x = 11, res_y = 220, res_val = 0.026645465
[2023-07-28T16:19:59Z INFO  yap::inference::img_process] res_x = 6, res_y = 180, res_val = 0.0063209254
[2023-07-28T16:19:59Z WARN  yap::pickupper::pickup_scanner] 黑名单: 新兵的徽记
[2023-07-28T16:19:59Z INFO  yap::inference::img_process] res_x = 6, res_y = 181, res_val = 0.00564668
[2023-07-28T16:19:59Z WARN  yap::pickupper::pickup_scanner] 黑名单: 新兵的徽记
[2023-07-28T16:19:59Z INFO  yap::inference::img_process] res_x = 7, res_y = 181, res_val = 0.006027725
[2023-07-28T16:19:59Z WARN  yap::pickupper::pickup_scanner] 黑名单: 新兵的徽记
[2023-07-28T16:20:00Z INFO  yap::inference::img_process] res_x = 7, res_y = 181, res_val = 0.0062591983
[2023-07-28T16:20:00Z INFO  yap::inference::img_process] res_x = 7, res_y = 181, res_val = 0.0061183823
[2023-07-28T16:20:00Z INFO  yap::inference::img_process] res_x = 7, res_y = 181, res_val = 0.005174378
[2023-07-28T16:20:00Z INFO  yap::inference::img_process] res_x = 6, res_y = 146, res_val = 0.004959859
[2023-07-28T16:20:00Z INFO  yap::inference::img_process] res_x = 6, res_y = 145, res_val = 0.0057055876
[2023-07-28T16:20:00Z INFO  yap::inference::img_process] res_x = 6, res_y = 145, res_val = 0.005873109
[2023-07-28T16:20:00Z INFO  yap::inference::img_process] res_x = 6, res_y = 145, res_val = 0.0018746178
[2023-07-28T16:20:00Z INFO  yap::inference::img_process] res_x = 7, res_y = 145, res_val = 0.0015709257
[2023-07-28T16:20:00Z INFO  yap::inference::img_process] res_x = 7, res_y = 145, res_val = 0.0028391497
[2023-07-28T16:20:01Z INFO  yap::inference::img_process] res_x = 0, res_y = 198, res_val = 0.090397656
[2023-07-28T16:20:01Z INFO  yap::pickupper::pickup_scanner] 拾起: 寻宝鸦印
[2023-07-28T16:20:01Z INFO  yap::inference::img_process] res_x = 11, res_y = 0, res_val = 0.09056256
[2023-07-28T16:20:01Z INFO  yap::inference::img_process] res_x = 11, res_y = 28, res_val = 0.090197496
[2023-07-28T16:20:01Z INFO  yap::inference::img_process] res_x = 0, res_y = 23, res_val = 0.08946642
[2023-07-28T16:20:01Z INFO  yap::inference::img_process] res_x = 0, res_y = 24, res_val = 0.08860656
```

gray
```
[2023-07-28T16:38:02Z INFO  yap::inference::img_process] res_x = 11, res_y = 136, res_val = 2.559872
[2023-07-28T16:38:02Z INFO  yap::inference::img_process] res_x = 0, res_y = 134, res_val = 2.397104
[2023-07-28T16:38:02Z INFO  yap::inference::img_process] res_x = 11, res_y = 132, res_val = 2.9388556
[2023-07-28T16:38:02Z INFO  yap::inference::img_process] res_x = 0, res_y = 0, res_val = 2.6888418
[2023-07-28T16:38:03Z INFO  yap::inference::img_process] res_x = 11, res_y = 124, res_val = 0.46062323
[2023-07-28T16:38:03Z INFO  yap::inference::img_process] res_x = 11, res_y = 129, res_val = 0.46035954
[2023-07-28T16:38:03Z INFO  yap::inference::img_process] res_x = 0, res_y = 126, res_val = 0.4447923
[2023-07-28T16:38:03Z INFO  yap::inference::img_process] res_x = 3, res_y = 301, res_val = 3.325155
[2023-07-28T16:38:03Z INFO  yap::inference::img_process] res_x = 0, res_y = 280, res_val = 1.0303878
[2023-07-28T16:38:03Z INFO  yap::inference::img_process] res_x = 0, res_y = 308, res_val = 0.6596797
[2023-07-28T16:38:03Z INFO  yap::inference::img_process] res_x = 0, res_y = 289, res_val = 1.0936271
[2023-07-28T16:38:03Z INFO  yap::inference::img_process] res_x = 8, res_y = 231, res_val = 1.6696731
[2023-07-28T16:38:03Z INFO  yap::inference::img_process] res_x = 8, res_y = 215, res_val = 2.76161
[2023-07-28T16:38:04Z INFO  yap::inference::img_process] res_x = 0, res_y = 11, res_val = 2.6334698
[2023-07-28T16:38:04Z INFO  yap::inference::img_process] res_x = 11, res_y = 89, res_val = 2.0683477
[2023-07-28T16:38:04Z INFO  yap::inference::img_process] res_x = 0, res_y = 308, res_val = 1.1541436
[2023-07-28T16:38:04Z INFO  yap::inference::img_process] res_x = 11, res_y = 313, res_val = 2.5101023
[2023-07-28T16:38:04Z INFO  yap::inference::img_process] res_x = 0, res_y = 342, res_val = 2.7891824
[2023-07-28T16:38:04Z INFO  yap::inference::img_process] res_x = 7, res_y = 334, res_val = 0.09020872
[2023-07-28T16:38:04Z INFO  yap::inference::img_process] res_x = 0, res_y = 265, res_val = 0.18991762
[2023-07-28T16:38:04Z INFO  yap::inference::img_process] res_x = 3, res_y = 180, res_val = 0.08078181
[2023-07-28T16:38:04Z INFO  yap::inference::img_process] res_x = 7, res_y = 110, res_val = 0.13069499
[2023-07-28T16:38:04Z INFO  yap::inference::img_process] res_x = 0, res_y = 107, res_val = 0.15565987
[2023-07-28T16:38:05Z INFO  yap::inference::img_process] res_x = 11, res_y = 156, res_val = 1.0413109
[2023-07-28T16:38:05Z INFO  yap::inference::img_process] res_x = 6, res_y = 157, res_val = 1.2102035
[2023-07-28T16:38:05Z INFO  yap::inference::img_process] res_x = 0, res_y = 158, res_val = 0.7170382
[2023-07-28T16:38:05Z INFO  yap::inference::img_process] res_x = 0, res_y = 156, res_val = 1.1217178
[2023-07-28T16:38:05Z INFO  yap::inference::img_process] res_x = 0, res_y = 149, res_val = 0.87846094
[2023-07-28T16:38:05Z INFO  yap::inference::img_process] res_x = 11, res_y = 328, res_val = 0.104632616
[2023-07-28T16:38:05Z INFO  yap::inference::img_process] res_x = 11, res_y = 342, res_val = 0.16651478
[2023-07-28T16:38:05Z INFO  yap::inference::img_process] res_x = 0, res_y = 221, res_val = 0.17508662
[2023-07-28T16:38:05Z INFO  yap::inference::img_process] res_x = 0, res_y = 210, res_val = 0.112228155
[2023-07-28T16:38:06Z INFO  yap::inference::img_process] res_x = 1, res_y = 227, res_val = 0.08619205
[2023-07-28T16:38:06Z INFO  yap::inference::img_process] res_x = 0, res_y = 241, res_val = 0.76836616
[2023-07-28T16:38:06Z INFO  yap::inference::img_process] res_x = 0, res_y = 0, res_val = 0.70655304
[2023-07-28T16:38:06Z INFO  yap::inference::img_process] res_x = 0, res_y = 293, res_val = 0.100716695
[2023-07-28T16:38:06Z INFO  yap::inference::img_process] res_x = 10, res_y = 34, res_val = 0.093881585
[2023-07-28T16:38:06Z INFO  yap::inference::img_process] res_x = 7, res_y = 309, res_val = 0.10702627
[2023-07-28T16:38:06Z INFO  yap::inference::img_process] res_x = 5, res_y = 320, res_val = 0.10731691
[2023-07-28T16:38:06Z INFO  yap::inference::img_process] res_x = 3, res_y = 62, res_val = 0.10891436
[2023-07-28T16:38:06Z INFO  yap::inference::img_process] res_x = 0, res_y = 342, res_val = 0.111002356
[2023-07-28T16:38:07Z INFO  yap::inference::img_process] res_x = 0, res_y = 106, res_val = 0.19409344
[2023-07-28T16:38:07Z INFO  yap::inference::img_process] res_x = 0, res_y = 342, res_val = 0.11561843
[2023-07-28T16:38:07Z INFO  yap::inference::img_process] res_x = 0, res_y = 140, res_val = 0.93572927
[2023-07-28T16:38:07Z INFO  yap::inference::img_process] res_x = 11, res_y = 135, res_val = 0.27766207
[2023-07-28T16:38:07Z INFO  yap::inference::img_process] res_x = 0, res_y = 0, res_val = 0.15929888
[2023-07-28T16:38:07Z INFO  yap::inference::img_process] res_x = 0, res_y = 194, res_val = 0.105995014
[2023-07-28T16:38:07Z INFO  yap::inference::img_process] res_x = 11, res_y = 128, res_val = 0.10362476
[2023-07-28T16:38:07Z INFO  yap::inference::img_process] res_x = 11, res_y = 12, res_val = 0.10331386
[2023-07-28T16:38:08Z INFO  yap::inference::img_process] res_x = 0, res_y = 27, res_val = 0.16549341
[2023-07-28T16:38:08Z INFO  yap::inference::img_process] res_x = 0, res_y = 27, res_val = 0.29879493
[2023-07-28T16:38:08Z INFO  yap::inference::img_process] res_x = 11, res_y = 125, res_val = 0.90877765
[2023-07-28T16:38:08Z INFO  yap::inference::img_process] res_x = 0, res_y = 227, res_val = 0.35799292
[2023-07-28T16:38:08Z INFO  yap::inference::img_process] res_x = 8, res_y = 277, res_val = 1.3126842
[2023-07-28T16:38:08Z INFO  yap::inference::img_process] res_x = 11, res_y = 18, res_val = 0.10479089
[2023-07-28T16:38:08Z INFO  yap::inference::img_process] res_x = 11, res_y = 328, res_val = 0.18273616
[2023-07-28T16:38:08Z INFO  yap::inference::img_process] res_x = 11, res_y = 342, res_val = 0.84665537
[2023-07-28T16:38:08Z INFO  yap::inference::img_process] res_x = 0, res_y = 301, res_val = 1.0870619
[2023-07-28T16:38:09Z INFO  yap::inference::img_process] res_x = 11, res_y = 310, res_val = 0.3271364
[2023-07-28T16:38:09Z INFO  yap::inference::img_process] res_x = 0, res_y = 0, res_val = 0.23271662
[2023-07-28T16:38:09Z INFO  yap::inference::img_process] res_x = 5, res_y = 27, res_val = 0.15846351
[2023-07-28T16:38:09Z INFO  yap::inference::img_process] res_x = 0, res_y = 39, res_val = 1.2849522
[2023-07-28T16:38:09Z INFO  yap::inference::img_process] res_x = 11, res_y = 177, res_val = 0.47979146
[2023-07-28T16:38:09Z INFO  yap::inference::img_process] res_x = 11, res_y = 342, res_val = 0.102415785
[2023-07-28T16:38:09Z INFO  yap::inference::img_process] res_x = 0, res_y = 108, res_val = 0.105764836
[2023-07-28T16:38:09Z INFO  yap::inference::img_process] res_x = 7, res_y = 0, res_val = 0.115409486
[2023-07-28T16:38:09Z INFO  yap::inference::img_process] res_x = 8, res_y = 78, res_val = 0.13348146
[2023-07-28T16:38:10Z INFO  yap::inference::img_process] res_x = 0, res_y = 35, res_val = 0.28648922
[2023-07-28T16:38:10Z INFO  yap::inference::img_process] res_x = 11, res_y = 295, res_val = 0.44481125
[2023-07-28T16:38:10Z INFO  yap::inference::img_process] res_x = 0, res_y = 342, res_val = 0.36055782
[2023-07-28T16:38:10Z INFO  yap::inference::img_process] res_x = 0, res_y = 246, res_val = 0.19230528
[2023-07-28T16:38:10Z INFO  yap::inference::img_process] res_x = 11, res_y = 110, res_val = 0.44917208
[2023-07-28T16:38:10Z INFO  yap::inference::img_process] res_x = 11, res_y = 87, res_val = 0.3392144
[2023-07-28T16:38:10Z INFO  yap::inference::img_process] res_x = 0, res_y = 267, res_val = 0.10047177
[2023-07-28T16:38:10Z INFO  yap::inference::img_process] res_x = 9, res_y = 271, res_val = 0.24630295
[2023-07-28T16:38:10Z INFO  yap::inference::img_process] res_x = 0, res_y = 12, res_val = 0.42904124
[2023-07-28T16:38:11Z INFO  yap::inference::img_process] res_x = 6, res_y = 125, res_val = 0.22845377
[2023-07-28T16:38:11Z INFO  yap::inference::img_process] res_x = 7, res_y = 109, res_val = 0.025985891
[2023-07-28T16:38:11Z INFO  yap::pickupper::pickup_scanner] 拾起: 失活菌核
[2023-07-28T16:38:11Z INFO  yap::inference::img_process] res_x = 7, res_y = 181, res_val = 0.057066653
[2023-07-28T16:38:11Z INFO  yap::pickupper::pickup_scanner] 拾起: 蕈兽孢子
[2023-07-28T16:38:11Z INFO  yap::inference::img_process] res_x = 6, res_y = 232, res_val = 0.0748256
[2023-07-28T16:38:11Z INFO  yap::pickupper::pickup_scanner] 拾起: 休眠菌核
[2023-07-28T16:38:11Z INFO  yap::inference::img_process] res_x = 11, res_y = 188, res_val = 0.2136465
[2023-07-28T16:38:11Z INFO  yap::inference::img_process] res_x = 11, res_y = 181, res_val = 0.8938781
[2023-07-28T16:38:11Z INFO  yap::inference::img_process] res_x = 11, res_y = 79, res_val = 0.8847677
[2023-07-28T16:38:11Z INFO  yap::inference::img_process] res_x = 11, res_y = 288, res_val = 1.662954
[2023-07-28T16:38:12Z INFO  yap::inference::img_process] res_x = 0, res_y = 190, res_val = 1.436075
[2023-07-28T16:38:12Z INFO  yap::inference::img_process] res_x = 11, res_y = 131, res_val = 1.2226018
[2023-07-28T16:38:12Z INFO  yap::inference::img_process] res_x = 0, res_y = 95, res_val = 1.9076998
[2023-07-28T16:38:12Z INFO  yap::inference::img_process] res_x = 11, res_y = 306, res_val = 0.49557522
[2023-07-28T16:38:12Z INFO  yap::inference::img_process] res_x = 0, res_y = 342, res_val = 0.22891214
[2023-07-28T16:38:12Z INFO  yap::inference::img_process] res_x = 0, res_y = 88, res_val = 0.340468
[2023-07-28T16:38:12Z INFO  yap::inference::img_process] res_x = 8, res_y = 5, res_val = 0.1806407
[2023-07-28T16:38:12Z INFO  yap::inference::img_process] res_x = 8, res_y = 5, res_val = 0.1806407
[2023-07-28T16:38:13Z INFO  yap::inference::img_process] res_x = 9, res_y = 216, res_val = 0.13005589
[2023-07-28T16:38:13Z INFO  yap::inference::img_process] res_x = 0, res_y = 331, res_val = 0.15834773
[2023-07-28T16:38:13Z INFO  yap::inference::img_process] res_x = 0, res_y = 331, res_val = 0.15834773
```

L通道更稳定，耗时较长