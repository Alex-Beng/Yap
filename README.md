<div align="center">

# Yap
Yet Another Genshin Impact Pickupper

又一个原神拾取器

_Named from [Yas](https://github.com/wormtql/yas)_

</div>

# 介绍

借鉴了[Yas](https://github.com/wormtql/yas)代码实现的自动拾取器。

一个开箱即用、跑的飞快、占用资源极低、可配置黑名单的自动拾取器，解放F键。


![pickup demo](./imgs/pk.gif)

![cpu](./imgs/cpu.PNG)




视频演示：[v0.1.0锄地](https://www.bilibili.com/video/BV1zk4y1G72J) [v0.1.1捡狗粮](https://www.bilibili.com/video/BV1ix4y197nv) 

模型训练：[yap-train](https://github.com/Alex-Beng/yap-train)

PS：旧版本模型精度（生成数据的问题，已解决）、推理间隔（100ms，现在是40ms）、滚动逻辑（目前是会上下翻的状态机）等问题，显得比较慢。

# 原理

与[Yas](https://github.com/wormtql/yas)一样，使用SVTR网络对文字进行识别。

对于F键的定位，使用基于灰度通道的模板匹配。


# 使用

目前仅支持windows。

## 从release获取

1. 点击[此处](https://github.com/Alex-Beng/Yap/releases)下载最新版本的release压缩包，解压。有两个文件：`yap.exe`和`black_lists.json`。

2. 使用记事本/VSCode等编辑器打开`black_lists.json`，添加需要拉黑的拾取物品名称，注意需要使用英文符号，如：


```json
[
    "史莱姆凝液",
    "污秽的面具",
    "破损的面具",
    "七天神像",
    "所有需要使用F键交互的对象"
]
```

3. 右键`yap.exe`选择以管理员身份运行



## 自行编译

1. 编译
```
cargo build --release
```

2. 修改`black_lists.json`，如上。

3. 确保`black_lists.json`在执行文件的同级目录下，如下所示。


```bash
yap> cargo run --release # 项目根目录有black_lists.json, work
```
当然你把黑名单加进环境变量也行，但不推荐。


4. 管理员运行/管理员身份打开终端运行

```bash
yap> ./target/release/yap.exe 
```
or 
```bash
yap> cargo run --release
```


5. 如果需要进行debug调试，可参考命令行参数：
```
/yap --help
YAP - 原神自动拾取器 0.1.2
Alex-Beng <pc98@qq.com>
Genshin Impact Pickup Helper

USAGE:
    yap.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --dump <dump>                                输出模型预测结果、原始图像、二值图像至指定的文件夹，debug专用
    -i, --dump-idx <dump_idx>                        执行dump时，输出结果起始的index [default: 0]
    -g, --infer-gap <infer_gap>                      一次检测推理拾取的间隔，单位ms [default: 40]
    -t, --template-threshold <template-threshold>    模板匹配的阈值，约小越严格 [default: 0.2]```
```

Just enjoy it!


# 已知问题 and TODO

1. 需要试验更好的F键定位，灰度通道精度太低
2. 更多的分辨率支持
3. ~~滚动逻辑优化~~（添加了上下滚动的状态机）
4. ~~丘丘游侠掉落物过长无法完全识别~~（添加了缩短的词）
5. 在较小的16:9分辨率（如720p）上会因为模板匹配而崩溃
6. 搜索F键与推理使用的是不同帧的画面（至少间隔20ms），infer-gap太低会不同步

# 总结

## 优点
1. 跑的快，单次推理速度低于10ms
2. 不占用GPU，使用CPU推理
3. 可执行文件体积小，加上CRNN模型仅10+MB
4. 更低的内存占用（对比各种python实现）
5. **开箱即用**，无需等待配置环境时的pip及github下载。
6. 专注于拾取，配置黑名单，解放F键

## 缺点
1. ~~编译速度太tmd慢了~~（无法解决，rust只有code gen并行）
2. ~~模型目前精度不佳，训练时使用了一半真实数据一半生成数据~~（生成数据接近真实数据，可以做到0 shot）
3. 目前仅支持16:9的分辨率
4. ~~拾取逻辑默认往下滚动，带来冗余~~
6. ~~因为使用中文识别，所以添加新字还需要改网络架构重新训练~~（同样无法解决，yas也是一样的）
7. F键寻找的算法（模板匹配）在灰度通道精度太低，需要尝试其他通道


相比拥有黑名单的自动拾取，如隔壁的[GIA](https://github.com/infstellar/genshin_impact_assistant)。rust带来了**更快**的运行速度（单次推理速度低于10ms vs 至少比10ms慢），**更小**的体积（14+MB vs 若干个GB），**更低**的内存占用（显然）。