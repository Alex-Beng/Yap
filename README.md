<div align="center">

# Yap
Yet Another Genshin Impact Pickupper

又一个原神拾取器

_Named from [Yas](https://github.com/wormtql/yas)_

</div>

# 介绍

借鉴了[Yas](https://github.com/wormtql/yas)代码实现的自动拾取器。

一个跑的飞快、占用资源低、可配置黑名单的自动拾取器，解放玩家的F键。


除了版本更新检测外，已基本完工。（可能也不会考虑去做另一个基于图像分类的方案了）

- [x] 1. 模型训练
- [x] 2. 模型转换
- [x] 4. F key 寻找
- [x] 10. F key Find api
- [ ] 5. version update detection
- [x] 6. Capture
- [x] 7. image Preprocess
- [x] 3. 模型部署及测试
- [x] 8. F key press
- [x] 9. Scroll press

### 优劣

优点
1. 跑的快，单次推理速度低于10ms
2. 不占用GPU，使用CPU推理
3. 可执行文件体积小，加上CRNN模型仅14+MB
4. 更低的内存占用（对比各种python实现）
5. **开箱即用**，无需等待配置环境时的pip及github下载。
6. 专注于拾取，配置黑名单，锄地玩家解放F键

缺点
1. 编译速度太tmd慢了
2. 模型目前精度不佳，训练时使用了一半真实数据一半生成数据
3. 目前仅支持16:9的分辨率
4. 拾取逻辑默认往下滚动，带来冗余
5. 跑的太快，两次检测之间硬编码了一个100ms的延迟


综上。相比拥有黑名单的自动拾取，如隔壁的[GIA](https://github.com/infstellar/genshin_impact_assistant)。rust带来了**更快**的运行速度（单次推理速度低于10ms vs 至少比10ms慢），**更小**的体积（14+MB vs 若干个GB），**更低**的内存占用（显然）。