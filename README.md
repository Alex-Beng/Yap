<div align="center">

# Yap
Yet Another Genshin Impact Pickupper

又一个原神拾取器

_Named from [Yas](https://github.com/wormtql/yas)_

</div>

# 介绍

借鉴了[Yas](https://github.com/wormtql/yas)代码实现的自动拾取器。

一个跑的飞快、占用资源低、可配置黑名单的自动拾取器，解放玩家的F键。

视频演示：[bilibili](https://www.bilibili.com/video/BV1zk4y1G72J)

# 原理

与[Yas](https://github.com/wormtql/yas)一眼，使用SVTR网络对文字进行识别。

对于F键的定位，使用基于灰度通道的模板匹配。

# 已知问题 and TODO

1. 需要试验更好的F键定位，灰度通道精度太低
2. 更多的分辨率支持
3. 滚动逻辑优化

# 使用

## 1. 获取可执行文件

下载 or 编译得到可执行文件

编译
```
cargo build --release
```

## 2. 配置黑名单

需要复制项目中的`black_lists.json`（见下图）至可执行文件同目录下。


将需要拉黑的拾取物品名称添加至`black_lists.json`中，如下所示。
```json
[
    "史莱姆凝液",
    "污秽的面具",
    "破损的面具",
    "七天神像",
    "所有需要使用F键交互的对象"
]
```

## 3. 运行

```bash
cargo run
```

如果需要进行debug调试，可参考命令行参数：
```
/yap --help
USAGE:
    yap.exe [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --dump <dump>              输出模型预测结果、原始图像、二值图像至指定的文件夹，debug专用
    -i, --dump-idx <dump_idx>      执行dump时，输出结果起始的index [default: 0]
    -g, --infer-gap <infer_gap>    一次检测推理拾取的间隔，单位ms [default: 100]
```


Just enjoy it!


# 优劣

## 优点
1. 跑的快，单次推理速度低于10ms
2. 不占用GPU，使用CPU推理
3. 可执行文件体积小，加上CRNN模型仅10+MB
4. 更低的内存占用（对比各种python实现）
5. **开箱即用**，无需等待配置环境时的pip及github下载。
6. 专注于拾取，配置黑名单，锄地玩家解放F键

## 缺点
1. 编译速度太tmd慢了
2. 模型目前精度不佳，训练时使用了一半真实数据一半生成数据
3. 目前仅支持16:9的分辨率
4. 拾取逻辑默认往下滚动，带来冗余
6. 因为使用中文识别，所以添加新字还需要改网络架构重新训练
7. F键寻找的算法（模板匹配）在灰度通道精度太低，需要尝试其他通道


相比拥有黑名单的自动拾取，如隔壁的[GIA](https://github.com/infstellar/genshin_impact_assistant)。rust带来了**更快**的运行速度（单次推理速度低于10ms vs 至少比10ms慢），**更小**的体积（14+MB vs 若干个GB），**更低**的内存占用（显然）。但是在模型精度方面还需要更多的“人工”智biao能zhu。