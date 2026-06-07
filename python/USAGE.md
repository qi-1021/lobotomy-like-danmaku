# Lobotomy Corporation Danmaku - 使用指南

## 安装

```bash
# 1. 创建 Python 虚拟环境
python3 -m venv /tmp/unityextract
source /tmp/unityextract/bin/activate

# 2. 安装依赖
pip install Pillow numpy -i https://pypi.tuna.tsinghua.edu.cn/simple

# 3. 确保 ffmpeg 已安装
brew install ffmpeg  # macOS
```

## 快速开始

### CLI 命令行

```bash
# 使用预设
python3 lobotomy_cli.py -i input.mp4 -o output.mp4 --preset core_suppression

# 自定义文字
python3 lobotomy_cli.py -i input.mp4 -o output.mp4 \
  --text "控制部" --color "#b43c3c" \
  --text "WARNING" --color "#cc3333"

# 使用 JSON 文件
python3 lobotomy_cli.py -i input.mp4 -o output.mp4 --texts texts.json

# 调整参数
python3 lobotomy_cli.py -i input.mp4 -o output.mp4 \
  --preset core_suppression \
  --density 0.5 \
  --max-active 6 \
  --size-min 20 \
  --size-max 40
```

### GUI 图形界面

```bash
python3 lobotomy_gui.py
# 或者使用启动脚本
./launch_gui.sh
```

## CLI 参数说明

### 输入输出
| 参数 | 说明 | 必需 |
|------|------|------|
| `-i, --input` | 输入视频文件 | ✓ |
| `-o, --output` | 输出视频文件 | 默认 output.mp4 |

### 文字输入（三选一）
| 参数 | 说明 |
|------|------|
| `--texts` | JSON 文件，格式: `[{"text":"控制部","color":"#b43c3c"}]` |
| `--preset` | 预设名称: `core_suppression`, `all_departments`, `minimal` |
| `--text` | 直接指定文字（可重复使用） |
| `--color` | 配合 --text 使用的颜色（默认 #b43c3c） |

### 生成参数
| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--density` | 0.45 | 密度 0.1(稀疏) ~ 1.0(密集) |
| `--max-active` | 4 | **最多同时显示的文字条数** |
| `--size-min` | 18 | 最小字体大小 |
| `--size-max` | 32 | 最大字体大小 |
| `--angle-min` | -14 | 最小旋转角度 |
| `--angle-max` | 14 | 最大旋转角度 |
| `--seed` | 42 | 随机种子（相同种子=相同结果） |

## 预设文字

### core_suppression
控制部核心抑制效果：CORE SUPPRESSION、控制部、MALKUTH、异想体已突破收容、CONTAINMENT BREACH、所有部门进入红色警戒、핵심 억제

### all_departments
所有部门名称：控制部、情报部、培训部、安保部、中央本部、福利部、惩戒部、记录部、研发部、构建部

### minimal
极简：WARNING、ALERT

## JSON 格式

```json
[
  {"text": "控制部", "color": "#b43c3c"},
  {"text": "情报部", "color": "#4eb0d8"},
  {"text": "CORE SUPPRESSION", "color": [180, 60, 60]}
]
```

支持的颜色格式：
- 十六进制: `"#b43c3c"`
- RGB 数组: `[180, 60, 60]`
- 逗号分隔: `"180,60,60"`

## GUI 功能

1. **导入视频** - 支持 mp4/mov/avi/mkv/webm
2. **文字编辑** - 手动输入或从预设选择
3. **颜色选择** - 8 色快捷按钮 + 自定义拾色器
4. **位置设置** - X/Y 坐标，或直接点击画布定位
5. **时间控制** - 开始/结束时间、透明度
6. **自动生成** - 一键生成随机叠加效果
7. **预览** - 实时预览当前时间点的文字效果
8. **导出** - 导出为 MP4 视频

### 快捷键
- `Ctrl+N` - 添加文字
- `Delete` - 删除选中
- `Ctrl+S` - 导出视频

## 文字放置规则

- 随机位置，避免屏幕边缘（防止裁字）
- 同时显示的文字不会重叠
- 每条文字显示 2.2~4.0 秒
- 渐显 (0.5秒) + 显示 + 渐隐 (0.4秒)

## 支持的字体

**中文**（自动选择 PingFang SC）：
- PingFang SC (macOS 系统字体，推荐)

**拉丁字母**（自动选择 Norwester）：
- Norwester (游戏原始字体)
