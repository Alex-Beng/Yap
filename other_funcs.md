# 隐藏功能

以下功能依赖16:9分辨率。

## 传送锚点右下出现 确认/传送 自动移动并点击

默认开启快捷键。

快捷键：`Alt + 9`，切换开启/关闭，默认关闭自动点击。

可通过`config.json`中的`click_tp`设置默认开启/关闭。


## 联机&进入世界自动摁Y

默认开启。

显示`秘境挑战组队邀请(x)s Y` or `进入世界申请(x)s Y`时，自动摁Y。

可通过`config.json`中的`press_y`设置默认开启/关闭。

## 强化圣遗物

默认开启快捷键。

快捷键：`Alt + B`。与主程序一样仅支持16:9。

固定动作，用切换`详情`的方式跳过强化动画。`点击 快捷放入 -> 点击 (右下）强化 -> 点击 详情 -> 点击 强化 -> 移动到 快捷放入`


## 调整拾取时序

需要添加hotkey命令行参数以开启快捷键。

| 快捷键 | 功能 |
| --- | --- |
| `Alt + J` | 增加`infer_gap` 1ms |
| `Alt + K` | 减少`infer_gap` 1ms |
| `Alt + U` | 增加`f_internal` 1ms |
| `Alt + I` | 减少`f_internal` 1ms |
| `Alt + L` | 增加`f_gap` 1ms |
| `Alt + H` | 减少`f_gap` 1ms |
| `Alt + O` | 增加`scroll_gap` 1ms |
| `Alt + P` | 减少`scroll_gap` 1ms |