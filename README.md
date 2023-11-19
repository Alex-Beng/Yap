<div align="center">

# Yap
Yet Another Genshin Impact Pickupper

又一个原神拾取器

_Named from [Yas](https://github.com/wormtql/yas)_

</div>

# 介绍

借鉴了[Yas](https://github.com/wormtql/yas)代码实现的自动拾取器。

一个开箱即用、跑的飞快、占用资源极低、可配置黑名单的自动拾取器，解放滚轮和F键，
Which may be the best open source pickupper in terms of performance, usability and configurability.

![pickup demo](./imgs/pk.gif)
![cpu](./imgs/cpu.PNG)




视频演示：[v0.1.0锄地](https://www.bilibili.com/video/BV1zk4y1G72J) [v0.1.1捡狗粮](https://www.bilibili.com/video/BV1ix4y197nv) 

模型训练：[yap-train](https://github.com/Alex-Beng/yap-train)

友情链接：[BetterGI--更好的原神，更多有用的功能](https://github.com/babalae/better-genshin-impact)

# 原理


使用~~基于L*/灰度通道的模板匹配（which is used in other naive pickuppers）~~基于轮廓提取+特征匹配的方案，实现μs级别的F键的定位；
通过固定位置关系截取拾取物的文字；
之后，与[Yas](https://github.com/wormtql/yas)一样，使用SVTR网络对预处理后的区域图片进行识别；


目前的策略是截取包含F键上下两个可能存在的拾取物文字，共五个区域。
然后根据黑白名单，利用硬编码的自上而下拾取算法，生成动作序列`ops`，再进行执行。

其中黑白名单的逻辑是：白名单中的物品必须拾取，黑名单中的物品若没有在白名单中，则不拾取。


整体流程的时序是：
```
... -> sleep infer_gap ms -> find F key -> infer image -> do pickup -> ...
or // 如果找不到F键
... -> sleep infer_gap ms -> find F key -> ...
```
其中可配置的`infer_gap`参数为推理间隔，单位ms。


拾取动作序列的时序是：
```
... F down ->  sleep f_internal ms -> F_up -> sleep f_gap ms -> ...
... scroll -> sleep scroll_gap ms -> ...
```
其中F键和滚轮的三个参数均可配置，分别为`f_internal`、`f_gap`、`scroll_gap`。

有两个子线程：
1. 用于监听全局快捷键，以暂停/恢复拾取，and other functions。
2. 用于检测进入世界&联机邀请，以自动摁下Y键。

知乎：[【原神】基于文字识别的超快自动拾取（有点老了，还是模板匹配做k键定位）](https://zhuanlan.zhihu.com/p/645909098)



# 使用

目前仅支持windows，PC客户端（已支持云原神），16:9分辨率。


全局快捷键见下表：

| 快捷键 | 功能 |
| --- | --- |
| `Alt + 0` | 暂停。切换是否拾取 |


隐藏功能见：[other_funcs.md](./other_funcs.md)

## 从release获取

目前有两种release，一种是由github actions自动构建的nightly版本，一种是手动构建的release版本。前者可能会有bug，后者较为稳定。

PS：可以使用nightly版本帮助debug。

1. 点击[此处](https://github.com/Alex-Beng/Yap/releases)下载release压缩包，解压。有三个文件：`yap.exe`、`black_lists.json`和`white_lists.json`。

2. 使用记事本/VSCode等编辑器打开`black_lists.json`，添加需要拉黑的拾取物品名称，注意需要使用**英文符号**，如：


```json
[
    "史莱姆凝液",
    "污秽的面具",
    "破损的面具",
    "七天神像",
    "所有会提示使用F键交互的对象"
]
```

3. 白名单`white_lists.json`设置同黑名单，不再赘述

4. 右键`yap.exe`选择以**管理员身份**运行


5. 性能调优（如果你会使用命令行设置参数的话）


可以通过修改`infer-gap`参数来调整推理间隔，即检测F键的间隔，单位ms。

默认为0ms。

对于60FPS游戏，一帧为16ms，但是拾取及滚动响应应该不是一帧完成的。

如果出现捡不起来，适当调高`f_internal`（拾取后等待时间），滚动不了，适当调高`scroll_gap`（滚动后等待时间）。

```bash
./yap.exe --infer-gap 16 # 推理间隔为16ms
./yap.exe -g 16 # 两种写法都可以
```


## 自行编译

1. 编译
```
cargo build --release
```

2. 修改`black_lists.json`及`white_lists.json`，如上。

3. 确保`black_lists.json`及`white_lists.json`在执行文件的同级目录下，如下所示。


```bash
yap> cargo run --release # 项目根目录有black_lists.json&white_lists.json, work
```
当然你把黑/白名单加进环境变量也行，但不推荐。


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
YAP - 原神自动拾取器 0.1.15
Alex-Beng <pc98@qq.com>
Genshin Impact Pickup Helper

USAGE:
    yap.exe [FLAGS] [OPTIONS] [hotkey]

FLAGS:
    -h, --help         Prints help information
        --no-pickup    不执行拾取，仅info拾取动作，debug专用
    -V, --version      Prints version information

OPTIONS:
    -c, --channal <channal>
            模板匹配时使用的通道，默认使用gray通道，另一个可选值为L*，推荐匹配阈值固定为0.01 [default: gray]

        --dump <dump>                                输出模型预测结果、原始图像、二值图像至指定的文件夹，debug专用
    -i, --dump-idx <dump_idx>                        执行dump时，输出结果起始的index [default: 0]
    -g, --infer-gap <infer_gap>                      一次检测推理拾取的间隔，单位ms [default: 45]
        --log <log>                                  日志等级，可选值为trace, debug, info, warn, error [default: warn]
    -t, --template-threshold <template-threshold>
            模板匹配的阈值，约小越严格，灰度通道中匹配值在0.01-0.09左右 [default: 0.08]


ARGS:
    <hotkey>    是否注册hotkey用于调整拾取时序，debug专用
```

Just enjoy it!


