# Lobotomy Corporation - Core Suppression Danmaku

**心机拉萨有，齐抛洒有，戏骨细泡有，内购泡有，那噶够洗泡有......**

模拟脑叶公司核心抑制效果的弹幕视频工具。将指定文字以随机位置、角度、大小叠加到视频上，支持打字机效果和渐显/渐隐。

## 特性

- 打字机效果：文字从中心向两边逐字蹦出
- 渐显/渐隐动画
- 随机位置、角度、大小
- 支持中文/英文/韩文
- GUI 图形界面 + CLI 命令行

## 安装

### 依赖

```bash
pip install -r requirements.txt
```

### ffmpeg

本工具依赖 ffmpeg 进行视频处理，请确保已安装：

```bash
# macOS
brew install ffmpeg

# Ubuntu/Debian
sudo apt install ffmpeg

# Windows (使用 scoop)
scoop install ffmpeg
```

### 字体

推荐将字体放入 `fonts_proper/` 目录（优先使用），未放入时会自动查找系统字体：

```bash
mkdir -p fonts_proper
# macOS
cp /System/Library/Fonts/PingFang.ttc fonts_proper/
# 或使用其他 CJK 字体（Noto Sans CJK、Microsoft YaHei 等）
```

自动查找顺序：
1. `fonts_proper/` 目录内的字体（优先）
2. 系统字体（macOS: PingFang, Windows: MSYaHei/SimHei, Linux: NotoSansCJK/WenQuanYi）

## 快速开始

### GUI 图形界面（推荐）

```bash
# macOS / Linux
./run_gui.sh

# Windows
双击 run_gui.bat
```

### CLI 命令行

```bash
# 使用 JSON 文件
python3 lobotomy_cli.py -i input.mp4 -o output.mp4 --texts texts.json

# 直接指定文字
python3 lobotomy_cli.py -i input.mp4 -o output.mp4 \
  --text "控制部" --color "#b43c3c" \
  --text "WARNING" --color "#cc3333"

# 调整参数
python3 lobotomy_cli.py -i input.mp4 -o output.mp4 \
  --texts texts.json \
  --density 0.5 \
  --max-active 6 \
  --size-min 20 \
  --size-max 40
```

## CLI 参数说明

### 输入输出
| 参数 | 说明 | 必需 |
|------|------|------|
| `-i, --input` | 输入视频文件 | ✓ |
| `-o, --output` | 输出视频文件 | 默认 output.mp4 |

### 文字输入
| 参数 | 说明 |
|------|------|
| `--texts` | JSON 文件，格式: `[{"text":"控制部","color":"#b43c3c"}]` |
| `--text` | 直接指定文字（可重复使用） |
| `--color` | 配合 --text 使用的颜色（默认 #b43c3c） |

### 生成参数
| 参数 | 默认值 | 说明 |
|------|--------|------|
| `--density` | 0.45 | 密度 0.1(稀疏) ~ 1.0(密集) |
| `--max-active` | 4 | 最多同时显示的文字条数 |
| `--size-min` | 18 | 最小字体大小 |
| `--size-max` | 32 | 最大字体大小 |
| `--angle-min` | -14 | 最小旋转角度 |
| `--angle-max` | 14 | 最大旋转角度 |
| `--seed` | 42 | 随机种子（相同种子=相同结果） |

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

## GUI 功能

1. **导入视频** - 支持 mp4/mov/avi/mkv/webm
2. **文字编辑** - 手动输入文字和颜色
3. **颜色选择** - 自定义拾色器
4. **位置设置** - X/Y 坐标，或直接点击画布定位
5. **时间控制** - 开始/结束时间、透明度、留存时间
6. **自动生成** - 一键生成随机叠加效果
7. **预览** - 实时预览当前时间点的文字效果
8. **导出** - 导出为 MP4 视频（保留原音频）
9. **项目保存/加载** - 保存和恢复项目配置

## 文字放置规则

- 随机位置，避免屏幕边缘（防止裁字）
- 同时显示的文字不会重叠
- 打字机效果：文字从中心向两边逐字蹦出
- 渐显 (0.4秒) + 打字 + 留存 + 渐隐 (0.3秒)

## 许可证

MIT License
