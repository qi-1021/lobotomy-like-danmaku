#!/usr/bin/env python3
"""
Core Suppression Danmaku Video Processor
Imports video, overlays random text at various positions/angles, exports result.
"""

import os, sys, json, math, random, subprocess, struct, platform
from collections import namedtuple
from PIL import Image, ImageDraw, ImageFont

VideoInfo = namedtuple('VideoInfo', ['width', 'height', 'fps', 'duration'])

# 内置字体目录（优先）
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
FONT_DIR = os.path.join(SCRIPT_DIR, "fonts_proper")

def _find_system_font(names):
    """在系统字体目录中查找字体（兜底）"""
    system = platform.system()
    if system == "Darwin":
        search_dirs = ["/System/Library/Fonts", "/Library/Fonts",
                       os.path.expanduser("~/Library/Fonts")]
    elif system == "Windows":
        search_dirs = [os.path.join(os.environ.get("WINDIR", "C:\\Windows"), "Fonts")]
    else:
        search_dirs = ["/usr/share/fonts", "/usr/local/share/fonts",
                       os.path.expanduser("~/.fonts"),
                       os.path.expanduser("~/.local/share/fonts")]
    
    for d in search_dirs:
        if not os.path.isdir(d):
            continue
        for root, _, files in os.walk(d):
            for name in names:
                for ext in ('.ttf', '.otf', '.ttc'):
                    target = name + ext
                    for f in files:
                        if f.lower() == target.lower():
                            return os.path.join(root, f)
    return None

def load_fonts():
    """Load fonts: bundled first, then system fallback"""
    fonts = {}
    # 1. 内置字体
    if os.path.isdir(FONT_DIR):
        for fname in os.listdir(FONT_DIR):
            if fname.lower().endswith(('.ttf', '.otf', '.ttc')):
                try:
                    fonts[fname] = ImageFont.truetype(os.path.join(FONT_DIR, fname), 24)
                except:
                    pass
    # 2. 系统字体兜底
    if not fonts:
        for name in ['PingFang', 'msyh', 'Microsoft YaHei', 'SimHei',
                      'NotoSansCJK', 'NotoSansSC', 'WenQuanYiMicroHei',
                      'Arial', 'Helvetica', 'DejaVuSans']:
            path = _find_system_font([name])
            if path:
                try:
                    fonts[os.path.basename(path)] = ImageFont.truetype(path, 24)
                except:
                    pass
    return fonts

def pick_font(text, font_size=24):
    """Pick best font: bundled first, system fallback"""
    has_cjk = any('\u4e00' <= c <= '\u9fff' or '\uac00' <= c <= '\ud7af' for c in text)
    
    # 1. 从内置目录找
    cjk_font = None
    latin_font = None
    if os.path.isdir(FONT_DIR):
        for fname in os.listdir(FONT_DIR):
            fl = fname.lower()
            if not fl.endswith(('.ttf', '.otf', '.ttc')):
                continue
            fpath = os.path.join(FONT_DIR, fname)
            if 'pingfang' in fl or 'noto' in fl or 'source' in fl \
                    or 'heiti' in fl or 'songti' in fl or 'hiragino' in fl \
                    or 'msyh' in fl or 'simhei' in fl or 'wqy' in fl:
                cjk_font = (fpath, 3 if 'pingfang' in fl else 0)
            elif 'norwester' in fl or 'arial' in fl or 'helvetica' in fl:
                latin_font = (fpath, 0)
            elif latin_font is None:
                latin_font = (fpath, 0)
    
    # 2. 系统字体兜底
    if cjk_font is None:
        sys_cjk = _find_system_font(['PingFang', 'STHeiti', 'Hiragino', 'msyh',
                                      'Microsoft YaHei', 'SimHei', 'NotoSansCJK',
                                      'NotoSansSC', 'WenQuanYiMicroHei'])
        if sys_cjk:
            idx = 3 if 'pingfang' in sys_cjk.lower() else 0
            cjk_font = (sys_cjk, idx)
    if latin_font is None:
        sys_latin = _find_system_font(['Helvetica', 'Arial', 'DejaVuSans',
                                        'LiberationSans', 'NotoSans'])
        if sys_latin:
            latin_font = (sys_latin, 0)
    
    if has_cjk and cjk_font:
        font_path, index = cjk_font
    elif latin_font:
        font_path, index = latin_font, 0
    elif cjk_font:
        font_path, index = cjk_font
    else:
        return ImageFont.load_default()
    
    try:
        return ImageFont.truetype(font_path, int(font_size), index=index)
    except:
        return ImageFont.load_default()

def probe_video(path):
    """Get video info via ffprobe"""
    out = subprocess.check_output([
        'ffprobe', '-v', 'quiet', '-print_format', 'json', '-show_streams', '-show_format', path
    ])
    info = json.loads(out)
    vs = next(s for s in info['streams'] if s['codec_type'] == 'video')
    w, h = int(vs['width']), int(vs['height'])
    rfr = vs.get('r_frame_rate', '30/1')
    parts = rfr.split('/')
    fps = float(parts[0]) / max(float(parts[1]), 1.0) if len(parts) == 2 else float(rfr)
    dur = float(info['format'].get('duration', '10'))
    return VideoInfo(w, h, fps, dur)

def render_frame(img, overlays, t, fonts):
    """Render text overlays with typewriter effect, characters expanding from center.

    Once a character appears, its position is fixed. New characters appear outward
    from the center. The text block rotates as a whole.
    """
    draw = ImageDraw.Draw(img)
    for o in overlays:
        s = o['start_time']
        e = o['end_time']
        a = o['alpha_max']

        if t < s or t > e:
            continue
        if t < s + 0.4:
            overall_alpha = min(1.0, (t - s) / 0.4) * a
        elif t > e - 0.3:
            overall_alpha = min(1.0, (e - t) / 0.3) * a
        else:
            overall_alpha = a

        if overall_alpha < 0.01:
            continue

        text = o['text']
        font_size = o['font_size']
        angle = o.get('angle', 0)

        # Typewriter: how many characters visible
        type_speed = o.get('type_speed', 0.06)
        elapsed = t - s
        chars_visible = min(len(text), int(elapsed / type_speed))
        if chars_visible <= 0:
            continue

        visible_text = text[:chars_visible]

        # Last char pop scale
        last_char_progress = (elapsed / type_speed) - chars_visible + 1
        pop_scale = 1.0
        if chars_visible < len(text) and last_char_progress < 1.0:
            pop_scale = 1.0 + 0.3 * math.sin(last_char_progress * math.pi)

        color = tuple(int(c * 255) for c in o['color'])
        alpha_int = int(overall_alpha * 255)
        fill = color + (alpha_int,) if img.mode == 'RGBA' else color

        # 创建字体对象（只创建两次：基础字号 + pop字号）
        base_font = pick_font(text, font_size)
        pop_font = pick_font(text, int(font_size * pop_scale)) if pop_scale > 1.0 else base_font

        # 测量已出现字符的宽度
        vis_char_widths = []
        for c in visible_text:
            cb = base_font.getbbox(c)
            vis_char_widths.append(cb[2] - cb[0])
        vis_total_w = sum(vis_char_widths)

        # 已出现字符的中心偏移
        vis_offsets = []
        x = 0
        for w in vis_char_widths:
            vis_offsets.append(x + w / 2 - vis_total_w / 2)
            x += w

        # 渲染到临时图（整行渲染，PIL 自然基线对齐）
        pad = max(30, int(font_size * 1.0))
        tmp_w = int(vis_total_w + pad * 2)
        tmp_h = int(font_size * 2.5 + pad * 2)
        tmp = Image.new('RGBA', (tmp_w, tmp_h), (0, 0, 0, 0))
        tmp_draw = ImageDraw.Draw(tmp)

        # 整行渲染（自然基线对齐，标点自动在正确位置）
        line_x = pad
        line_y = pad + (tmp_h - font_size) / 2
        tmp_draw.text((int(line_x), int(line_y)), visible_text, font=base_font, fill=fill)

        # 如果最后一个字符有 pop 效果，单独重绘
        if chars_visible < len(text):
            last_char = visible_text[-1]
            last_cb = base_font.getbbox(last_char)
            last_w = last_cb[2] - last_cb[0]
            last_x = line_x + vis_total_w - last_w
            # 清除旧位置
            tmp_draw.rectangle([int(last_x - 2), int(line_y - 2),
                                int(last_x + last_w + 2), int(line_y + font_size + 2)],
                               fill=(0, 0, 0, 0))
            tmp_draw.text((int(last_x), int(line_y)), last_char, font=pop_font, fill=fill)

        # 整体旋转
        if abs(angle) > 0.5:
            tmp = tmp.rotate(angle, expand=True, resample=Image.BICUBIC)

        # 粘贴到主图（居中于 o['x'], o['y']）
        paste_x = int(o['x'] - tmp.width / 2)
        paste_y = int(o['y'] - tmp.height / 2)
        if img.mode == 'RGBA':
            img.paste(tmp, (paste_x, paste_y), tmp)
        else:
            img.paste(tmp.convert('RGB'), (paste_x, paste_y))

def measure_text_box(text, font_size, angle, fonts):
    font = pick_font(text, font_size)
    bbox = font.getbbox(text)
    w = max(1, bbox[2] - bbox[0])
    h = max(1, bbox[3] - bbox[1])
    # Expand to the rotated bounding box plus padding.
    rad = math.radians(abs(angle))
    rw = abs(w * math.cos(rad)) + abs(h * math.sin(rad))
    rh = abs(w * math.sin(rad)) + abs(h * math.cos(rad))
    pad = max(24, font_size * 0.9)
    return rw + pad * 2, rh + pad * 2

def estimate_text_box(text, font_size):
    # Conservative approximation used only for random placement / overlap checks.
    # CJK characters are roughly square at their font size; Latin is narrower.
    cjk_count = sum(1 for c in text if '\u4e00' <= c <= '\u9fff' or '\uac00' <= c <= '\ud7af')
    latin_count = max(0, len(text) - cjk_count)
    # CJK: each char ≈ font_size wide. Latin: ~0.55 of font_size.
    width = cjk_count * font_size * 1.1 + latin_count * font_size * 0.55
    height = font_size * 1.4
    return width, height

def intersects(a, b, pad=18):
    ax0, ay0, ax1, ay1 = a
    bx0, by0, bx1, by1 = b
    return not (ax1 + pad < bx0 or bx1 + pad < ax0 or ay1 + pad < by0 or by1 + pad < ay0)

def make_overlays(texts, duration, width, height, *, density=0.45, max_active=4, seed=42,
                  size_min=18, size_max=32, angle_min=-14, angle_max=14,
                  type_speed=0.06, post_hold=1.5, alpha_max=0.7, fonts=None):
    """Generate controlled static text overlays.

    Each text from the input list can appear multiple times throughout the video.
    The total number of overlays is controlled by density and duration.
    """
    random.seed(seed)
    overlays = []
    if not texts:
        return overlays

    # 计算预期叠加数量：密度越高、时长越长，叠加越多
    estimated_count = max(len(texts), int(duration * density * 3))

    step = max(0.6, 1.6 - density)
    t = 0.4
    while t < duration - 0.8 and len(overlays) < estimated_count * 2:
        active = [o for o in overlays if o['start_time'] <= t <= o['end_time']]
        if len(active) >= max_active:
            t += 0.2
            continue

        # 随机从文字列表中选一个（允许重复）
        text, color = random.choice(texts)
        color_float = (color[0]/255.0, color[1]/255.0, color[2]/255.0)
        font_size = random.randint(size_min, size_max)
        angle = random.uniform(angle_min, angle_max)
        if fonts:
            box_w, box_h = measure_text_box(text, font_size, angle, fonts)
        else:
            box_w, box_h = estimate_text_box(text, font_size)
        margin_x = max(40, box_w * 0.65)
        margin_y = max(36, box_h * 1.2)

        x = y = None
        rect = None
        for _ in range(60):
            px = random.uniform(margin_x, width - margin_x)
            py = random.uniform(margin_y, height - margin_y)
            candidate = (px - box_w/2, py - box_h/2, px + box_w/2, py + box_h/2)
            if not any(intersects(candidate, o['rect']) for o in active if 'rect' in o):
                x, y, rect = px, py, candidate
                break
        if x is None:
            t += 0.2
            continue

        start = t
        # 显示时间 = 打字时间 + 留存时间 + 淡出缓冲
        type_duration = len(text) * type_speed
        min_display = type_duration + post_hold + 0.5
        display = max(min_display, random.uniform(2.5, 4.5))
        end = min(start + display, duration)
        alpha = max(0.15, min(1.0, alpha_max + random.uniform(-0.12, 0.12)))
        overlays.append({
            'text': text, 'font_size': font_size, 'color': color_float,
            'x': x, 'y': y, 'angle': angle,
            'start_time': start, 'end_time': end, 'alpha_max': alpha,
            'type_speed': type_speed, 'post_hold': post_hold, 'rect': rect,
        })
        # 低于 max_active 时加快生成速度，确保能同时达到上限
        current_active = sum(1 for o in overlays if o['start_time'] <= t <= o['end_time'])
        if current_active < max_active - 1:
            t += random.uniform(0.1, 0.3)  # 快速填充
        else:
            t += random.uniform(step * 0.75, step * 1.35)
    return overlays

def parse_color(value):
    value = value.strip()
    if value.startswith('#'):
        value = value[1:]
    if len(value) == 6:
        return tuple(int(value[i:i+2], 16) for i in (0, 2, 4))
    parts = [int(p.strip()) for p in value.split(',')]
    if len(parts) != 3:
        raise ValueError(f"Invalid color: {value}")
    return tuple(parts)

def load_text_specs(path):
    """Load user-provided text/color specs.

    Supported JSON formats:
    [{"text":"控制部", "color":"#b43c3c"}, ...]
    [{"text":"控制部", "color":[180,60,60]}, ...]
    [["控制部", "#b43c3c"], ...]
    """
    with open(path, encoding='utf-8') as f:
        data = json.load(f)
    specs = []
    for item in data:
        if isinstance(item, dict):
            text = str(item['text'])
            color = item.get('color', [180, 175, 190])
        else:
            text, color = item[0], item[1]
        if isinstance(color, str):
            color = parse_color(color)
        else:
            color = tuple(int(c) for c in color)
        specs.append((text, color))
    return specs

def process_video(input_path, output_path, overlays, fonts, progress_cb=None):
    """Decode video, overlay text, encode output, preserving audio.
    progress_cb(current_frame, total_frames) called periodically.
    """
    w, h, fps, dur = probe_video(input_path)
    total_frames = int(dur * fps)
    print(f"Video: {w}x{h} @ {fps:.1f}fps, {dur:.1f}s, {total_frames} frames")

    # Step 1: 渲染视频帧到临时文件
    import tempfile
    tmp_video = tempfile.mktemp(suffix='.mp4')

    decoder = subprocess.Popen([
        'ffmpeg', '-i', input_path, '-f', 'rawvideo', '-pix_fmt', 'rgba',
        '-s', f'{w}x{h}', '-an', '-'
    ], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)

    encoder = subprocess.Popen([
        'ffmpeg', '-y', '-f', 'rawvideo', '-pix_fmt', 'rgba',
        '-s', f'{w}x{h}', '-r', str(fps),
        '-i', '-', '-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-crf', '18',
        '-v', 'quiet', tmp_video
    ], stdin=subprocess.PIPE, stderr=subprocess.DEVNULL)

    frame_size = w * h * 4
    frame_num = 0

    while True:
        raw = decoder.stdout.read(frame_size)
        if len(raw) < frame_size:
            break

        t = frame_num / fps
        img = Image.frombytes('RGBA', (w, h), raw)
        render_frame(img, overlays, t, fonts)
        try:
            encoder.stdin.write(img.tobytes())
        except BrokenPipeError:
            break

        frame_num += 1
        if frame_num % 30 == 0:
            print(f"  Frame {frame_num}/{total_frames}")
            if progress_cb:
                progress_cb(frame_num, total_frames)
        if frame_num >= total_frames:
            break

    decoder.kill()
    encoder.stdin.close()
    encoder.wait()

    # Step 2: 合并原视频音频
    try:
        # 检查原视频是否有音轨
        has_audio = subprocess.run([
            'ffprobe', '-i', input_path, '-select_streams', 'a',
            '-show_entries', 'stream=codec_type', '-v', 'quiet', '-of', 'csv=p=0'
        ], capture_output=True, text=True).stdout.strip()

        if has_audio == 'audio':
            subprocess.run([
                'ffmpeg', '-y',
                '-i', tmp_video,
                '-i', input_path,
                '-c:v', 'copy', '-c:a', 'aac', '-map', '0:v:0', '-map', '1:a:0',
                '-shortest', '-v', 'quiet', output_path
            ], check=True)
        else:
            os.replace(tmp_video, output_path)
    except Exception:
        os.replace(tmp_video, output_path)
    finally:
        if os.path.exists(tmp_video):
            os.remove(tmp_video)

    if progress_cb:
        progress_cb(total_frames, total_frames)
    print(f"Done! Output: {output_path}")

def main():
    import argparse
    parser = argparse.ArgumentParser(description='Core Suppression Danmaku Video Processor')
    parser.add_argument('-i', '--input', required=True, help='Input video')
    parser.add_argument('-o', '--output', default='output.mp4', help='Output video')
    parser.add_argument('--duration', type=float, help='Override duration')
    parser.add_argument('--json', help='Explicit overlay JSON file')
    parser.add_argument('--texts', help='User text/color JSON file')
    parser.add_argument('--density', type=float, default=0.45, help='0.1 sparse, 1.0 denser')
    parser.add_argument('--max-active', type=int, default=4, help='Maximum simultaneous visible texts')
    parser.add_argument('--size-min', type=int, default=18)
    parser.add_argument('--size-max', type=int, default=32)
    parser.add_argument('--angle-min', type=float, default=-14)
    parser.add_argument('--angle-max', type=float, default=14)
    parser.add_argument('--seed', type=int, default=42)
    args = parser.parse_args()

    fonts = load_fonts()
    print(f"Loaded {len(fonts)} fonts")

    w, h, fps, dur = probe_video(args.input)
    if args.duration:
        dur = args.duration

    if args.json:
        with open(args.json) as f:
            overlay_data = json.load(f)
        overlays = []
        for o in overlay_data:
            overlays.append(o)
    else:
        text_specs = load_text_specs(args.texts) if args.texts else []
        overlays = make_overlays(
            text_specs, dur, w, h,
            density=args.density,
            max_active=args.max_active,
            seed=args.seed,
            size_min=args.size_min,
            size_max=args.size_max,
            angle_min=args.angle_min,
            angle_max=args.angle_max,
            fonts=fonts,
        )

    print(f"Overlays: {len(overlays)}")
    process_video(args.input, args.output, overlays, fonts)

if __name__ == '__main__':
    main()
