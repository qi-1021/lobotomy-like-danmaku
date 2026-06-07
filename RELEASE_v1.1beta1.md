## v1.1beta1 - GUI + 启动脚本

### 新功能
- **GUI 图形界面**（tkinter）- 双击启动，无需输代码
- **启动脚本** - run_gui.sh (macOS/Linux) / run_gui.bat (Windows)
- **手动编辑面板** - 完整参数控制（文字/字号/角度/位置/颜色/时间/速度/透明度/留存）
- **项目保存/加载** - 保存和恢复项目配置
- **自动透明度参数** - 生成时可设置透明度
- **颜色选择器** - 显示当前颜色值

### 改进
- 支持内置字体（fonts_proper/ 目录）
- 删除所有预设文字，要求显式输入
- 滚轮支持 macOS 触控板
- 左侧初始宽度增加，确保拖拽条可见

### 修复
- 修复打字机效果标点垂直对齐
- 修复旋转文字边缘裁字问题
- 修复导出视频音频保留
- 修复手动编辑面板缺少透明度/留存时间参数

### 使用方法
1. 安装依赖：`pip install -r requirements.txt`
2. 准备字体：`mkdir fonts_proper && cp /System/Library/Fonts/PingFang.ttc fonts_proper/`
3. 启动 GUI：`./run_gui.sh` 或双击 `run_gui.bat`
