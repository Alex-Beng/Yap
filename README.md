<div align="center">

# Yap
Yet Another Genshin Impact Pickupper

又一个原神拾取器

_Named from [Yas](https://github.com/wormtql/yas)_

</div>

# 介绍

借鉴了[Yas](https://github.com/wormtql/yas)代码实现的自动拾取器。

一个开箱即用、跑的飞快、占用资源极低、可配置黑名单的自动拾取器，解放滚轮和F键。


![pickup demo](./imgs/pk.gif)

![cpu](./imgs/cpu.PNG)




视频演示：[v0.1.0锄地](https://www.bilibili.com/video/BV1zk4y1G72J) [v0.1.1捡狗粮](https://www.bilibili.com/video/BV1ix4y197nv) 

模型训练：[yap-train](https://github.com/Alex-Beng/yap-train)


PS：旧版本（≤v0.1.5）为单区域策略，导致性能受制于推理间隔`infer-gap`及模板匹配+推理的时间，
在v0.1.6中改为多区域策略，性能不再受制于`infer-gap`。

PS：旧版本模型精度（生成数据的问题，已解决）、推理间隔（100ms，现在是60ms）、滚动逻辑（目前是会上下翻的状态机）等问题，显得比较慢。

# 原理


使用基于L*/灰度通道的模板匹配进行F键的定位，使用固定位置截取拾取物的文字。
目前的策略是截取包含F键上下两个可能存在的拾取物文字，共五个区域。

之后，与[Yas](https://github.com/wormtql/yas)一样，使用SVTR网络对预处理后的区域图片进行识别。

通过一组硬编码的自上而下拾取逻辑，生成动作序列`ops`，再进行执行。

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

使用一个子线程监听全局快捷键，以配置参数。

知乎：[【原神】基于文字识别的超快自动拾取](https://zhuanlan.zhihu.com/p/645909098)



# 使用

目前仅支持windows，常规PC客户端（已支持云原神），16:9分辨率。


全局快捷键见下表：

| 快捷键 | 功能 |
| --- | --- |
| `Alt + F` | 暂停。切换是否拾取 |
| `Alt + J` | 增加`infer_gap` 1ms |
| `Alt + K` | 减少`infer_gap` 1ms |
| `Alt + U` | 增加`f_internal` 1ms |
| `Alt + I` | 减少`f_internal` 1ms |
| `Alt + L` | 增加`f_gap` 1ms |
| `Alt + H` | 减少`f_gap` 1ms |
| `Alt + O` | 增加`scroll_gap` 1ms |
| `Alt + P` | 减少`scroll_gap` 1ms |

PS：一般情况下只需要使用`Alt + F`，默认参数基本稳定。

隐藏功能见：[other_funcs.md](./other_funcs.md)

## 从release获取

1. 点击[此处](https://github.com/Alex-Beng/Yap/releases)下载最新版本的release压缩包，解压。有三个文件：`yap.exe`、`black_lists.json`和`white_lists.json`。

2. 使用记事本/VSCode等编辑器打开`black_lists.json`，添加需要拉黑的拾取物品名称，注意需要使用英文符号，如：


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

4. 右键`yap.exe`选择以管理员身份运行


5. 性能调优（如果你会使用命令行设置参数的话）


可以通过修改`infer-gap`参数来调整推理间隔，单位ms。

默认值为60ms。

对于60FPS游戏，一帧为16ms，但是拾取响应应该不是一帧完成的。如果出现不同步，适当调高或调低，默认60ms，最低值16ms。

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
YAP - 原神自动拾取器 0.1.6
Alex-Beng <pc98@qq.com>
Genshin Impact Pickup Helper

USAGE:
    yap.exe [FLAGS] [OPTIONS]

FLAGS:
    -h, --help         Prints help information
        --no-pickup    不执行拾取，仅info拾取动作，debug专用
    -V, --version      Prints version information

OPTIONS:
    -c, --channal <channal>
            模板匹配时使用的通道，默认使用gray通道，另一个可选值为L*，推荐匹配阈值固定为0.01 [default: gray]

        --dump <dump>                                输出模型预测结果、原始图像、二值图像至指定的文件夹，debug专用
    -i, --dump-idx <dump_idx>                        执行dump时，输出结果起始的index [default: 0]
    -g, --infer-gap <infer_gap>                      一次检测推理拾取的间隔，单位ms [default: 0]
        --log <log>                                  日志等级，可选值为trace, debug, info, warn, error [default: warn]
    -t, --template-threshold <template-threshold>
            模板匹配的阈值，约小越严格，灰度通道中匹配值在0.01-0.09左右 [default: 0.1]
```

Just enjoy it!


# 已知问题 and TODO

1. ~~需要试验更好的F键定位，灰度通道匹配太多flase positive。~~（添加了L通道支持，没有显著性能差别）
2. 更多的分辨率支持
3. ~~滚动逻辑优化~~（添加了上下滚动的状态机）
4. ~~丘丘游侠掉落物过长无法完全识别~~（添加了缩短的词）
5. ~~在较小的16:9分辨率（如720p）上会因为模板匹配而崩溃~~（因为没对template图片缩放，我是傻逼，所以也是灰度通道效果不佳部分原因，已解决）
6. ~~搜索F键与推理使用的可能为（至少间隔16ms的）不同帧的画面，infer-gap太低会不同步~~（改为同一张了，但极限gap下可能还会不同步）
7. clean up code
8. ~~总是检测到启动器而不是本体的窗口~~（stolen from yas，检测窗口class）
9. 云原神支持
10. ~~重写状态机~~（v0.1.6）
11. ~~一次感知所有text~~（v0.1.6）
12. 模板匹配耗时太长，`~50ms`。推理也才`~30ms`。改用网络？
13. ~~4.0 新材料~~ (v0.1.9)
14. ~~白名单支持~~(v0.1.9)
15. 添加暂停支持，一直摁F太蠢了
16. ~~typo `初生的浊水幻灵` & `出生的浊水幻灵`，需要重新训练~~（v0.1.10）
17. 强光场景，如枫丹水下，阈值化效果差
18. 将known problems & TODO移动到github issue

# 总结

## 优点
1. **开箱即用**，无需等待配置环境时的pip及github下载。
2. 跑的快，单次推理速度低于10ms
3. 不占用GPU，使用CPU推理
4. 可执行文件体积小，加上CRNN模型仅10+MB
5. 更低的内存占用（对比各种python实现）
6. 专注于拾取，配置黑名单，解放F键

## 缺点
1. ~~编译速度太tmd慢了~~（无法解决，rust只有code gen并行）
2. ~~模型目前精度不佳，训练时使用了一半真实数据一半生成数据~~（生成数据接近真实数据，可以做到0 shot）
3. 目前仅支持16:9的分辨率
4. ~~拾取逻辑默认往下滚动，带来冗余~~
6. ~~因为使用中文识别，所以添加新字还需要改网络架构重新训练~~（同样无法解决，yas也是一样的）
7. ~~F键寻找的算法（模板匹配）在灰度通道精度较低，需要尝试其他通道~~


相比拥有黑名单的自动拾取，如隔壁的[GIA](https://github.com/infstellar/genshin_impact_assistant)。rust带来了**更快**的运行速度（单次推理速度低于10ms vs 至少比10ms慢），**更小**的体积（14+MB vs 若干个GB），**更低**的内存占用（显然）。