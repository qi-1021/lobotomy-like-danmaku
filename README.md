# Lobotomy Corporation - Core Suppression Danmaku (Rust)

**鬼知道会有什么bug，反正我要被AI搞崩溃了。**

**要是那些AI有部长一半好用，我早就解脱了。**

**附：sbAI还把吐槽删了，多有意思！还好我留了备份。**

Rust 原生版本，模拟脑叶公司核心抑制效果的弹幕视频工具。

## 构建

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 构建
cargo build --release
```

## 使用

### GUI（推荐）

```bash
# macOS / Linux
./run_rust.sh

# Windows
run_rust.bat
```

GUI 功能：
- 视频导入 + 实时预览（帧解码 + 叠加渲染）
- 文字列表管理（添加/删除/清空/改颜色）
- 颜色选择器（RGB 滑块 + HEX 输入 + 8 色快捷）
- 生成参数调节（密度/并发/字号/角度/速度/留存/透明度）
- 叠加列表（选中/删除/清空）
- 手动编辑（全部参数）
- 导出视频（进度条 + 百分比）
- 保存/加载项目 JSON
- 导出/导入预设
- 画布点击获取坐标
- Delete 键快捷删除

### CLI

```bash
# 使用 JSON 文件
./target/release/danmaku -i input.mp4 -o output.mp4 --texts texts.json

# 直接指定文字
./target/release/danmaku -i input.mp4 -o output.mp4 \
  --text "控制部" --color "#b43c3c" \
  --text "WARNING" --color "#cc3333"

# 调整参数
./target/release/danmaku -i input.mp4 -o output.mp4 \
  --text "控制部" --text "WARNING" \
  --density 0.5 --max-active 6
```

### CLI 参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `-i` | (必填) | 输入视频 |
| `-o` | output.mp4 | 输出视频 |
| `--texts` | - | JSON 文字文件 |
| `--text` | - | 直接添加文字（可重复） |
| `--color` | #b43c3c | 文字颜色 |
| `--density` | 0.45 | 密度 0.1-1.0 |
| `--max-active` | 4 | 最大同时显示数 |
| `--size-min` | 18 | 最小字号 |
| `--size-max` | 32 | 最大字号 |
| `--angle-min` | -14 | 最小角度 |
| `--angle-max` | 14 | 最大角度 |
| `--type-speed` | 0.06 | 打字速度（秒/字） |
| `--post-hold` | 1.5 | 留存时间（秒） |
| `--seed` | 42 | 随机种子 |

## JSON 格式

```json
[
  {"text": "控制部", "color": "#b43c3c"},
  {"text": "情报部", "color": [78, 176, 216]}
]
```

## 依赖

- Rust 1.70+
- ffmpeg（视频处理）

## 项目结构

```
crates/
├── danmaku-core/    # 核心引擎（渲染、叠加生成、视频处理、TTC 字体支持）
├── danmaku-cli/     # CLI 工具
└── danmaku-gui/     # GUI（eframe/egui，已完成）
```

