#!/usr/bin/env python3
"""
Lobotomy Corporation Danmaku CLI
Usage:
  python3 lobotomy_cli.py -i input.mp4 -o output.mp4 --texts texts.json
  python3 lobotomy_cli.py -i input.mp4 -o output.mp4 --preset core_suppression
  python3 lobotomy_cli.py -i input.mp4 -o output.mp4 --text "控制部" --color "#b43c3c"
"""

import argparse, json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from danmaku_processor import (
    load_fonts, probe_video, make_overlays, process_video,
    load_text_specs, parse_color, FONT_DIR, PINGFANG_SC
)

PRESETS = {
    "core_suppression": [
        {"text": "CORE SUPPRESSION", "color": "#b43c3c"},
        {"text": "控制部", "color": "#b43c3c"},
        {"text": "MALKUTH", "color": "#4eb0d8"},
        {"text": "异想体已突破收容", "color": "#b8b3c8"},
        {"text": "CONTAINMENT BREACH", "color": "#9b9b32"},
        {"text": "所有部门进入红色警戒", "color": "#b43c3c"},
        {"text": "핵심 억제", "color": "#b43c3c"},
    ],
    "all_departments": [
        {"text": "控制部", "color": "#b43c3c"},
        {"text": "情报部", "color": "#4eb0d8"},
        {"text": "培训部", "color": "#4ebd6e"},
        {"text": "安保部", "color": "#a54bbe"},
        {"text": "中央本部", "color": "#ccb444"},
        {"text": "福利部", "color": "#4eb0d8"},
        {"text": "惩戒部", "color": "#b43c3c"},
        {"text": "记录部", "color": "#a54bbe"},
        {"text": "研发部", "color": "#4ebd6e"},
        {"text": "构建部", "color": "#ccb444"},
    ],
    "minimal": [
        {"text": "WARNING", "color": "#b43c3c"},
        {"text": "ALERT", "color": "#9b9b32"},
    ],
}

def main():
    parser = argparse.ArgumentParser(
        description="Lobotomy Corporation Danmaku Generator",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Presets: core_suppression, all_departments, minimal

Examples:
  %(prog)s -i in.mp4 -o out.mp4 --preset core_suppression
  %(prog)s -i in.mp4 -o out.mp4 --text "控制部" --color "#b43c3c"
  %(prog)s -i in.mp4 -o out.mp4 --texts my_texts.json --density 0.5 --max-active 6
        """,
    )
    parser.add_argument("-i", "--input", required=True, help="Input video file")
    parser.add_argument("-o", "--output", default="output.mp4", help="Output video file")

    text_group = parser.add_argument_group("Text input (choose one)")
    text_group.add_argument("--texts", help="JSON file with text/color specs")
    text_group.add_argument("--preset", choices=list(PRESETS.keys()), help="Use a preset text list")
    text_group.add_argument("--text", action="append", default=[], help="Add text (can repeat)")
    text_group.add_argument("--color", default="#b43c3c", help="Color for --text (hex or R,G,B)")

    gen_group = parser.add_argument_group("Generation parameters")
    gen_group.add_argument("--density", type=float, default=0.45, help="0.1=sparse, 1.0=dense (default: 0.45)")
    gen_group.add_argument("--max-active", type=int, default=4, help="Max simultaneous texts (default: 4)")
    gen_group.add_argument("--size-min", type=int, default=18, help="Min font size (default: 18)")
    gen_group.add_argument("--size-max", type=int, default=32, help="Max font size (default: 32)")
    gen_group.add_argument("--angle-min", type=float, default=-14, help="Min rotation angle (default: -14)")
    gen_group.add_argument("--angle-max", type=float, default=14, help="Max rotation angle (default: 14)")
    gen_group.add_argument("--type-speed", type=float, default=0.06, help="Seconds per character, 0.02=fast, 0.1=slow (default: 0.06)")
    gen_group.add_argument("--post-hold", type=float, default=1.5, help="Seconds to hold after typing completes (default: 1.5)")
    gen_group.add_argument("--seed", type=int, default=42, help="Random seed (default: 42)")

    args = parser.parse_args()

    if not os.path.exists(args.input):
        print(f"Error: Input file not found: {args.input}", file=sys.stderr)
        sys.exit(1)

    # Build text specs
    if args.texts:
        text_specs = load_text_specs(args.texts)
    elif args.preset:
        text_specs = [(t["text"], parse_color(t["color"])) for t in PRESETS[args.preset]]
    elif args.text:
        color = parse_color(args.color)
        text_specs = [(t, color) for t in args.text]
    else:
        print("Error: Must specify --texts, --preset, or --text", file=sys.stderr)
        sys.exit(1)

    if not text_specs:
        print("Error: No texts specified", file=sys.stderr)
        sys.exit(1)

    # Load fonts and probe video
    fonts = load_fonts()
    w, h, fps, dur = probe_video(args.input)

    print(f"Input:    {args.input} ({w}x{h} @ {fps:.1f}fps, {dur:.1f}s)")
    print(f"Output:   {args.output}")
    print(f"Texts:    {len(text_specs)} entries")
    for text, color in text_specs:
        print(f"  - {text} ({color})")
    print(f"Settings: density={args.density}, max-active={args.max_active}, "
          f"size={args.size_min}-{args.size_max}, angle={args.angle_min}~{args.angle_max}, "
          f"type-speed={args.type_speed}s/char")
    print()

    # Generate overlays
    overlays = make_overlays(
        text_specs, dur, w, h,
        density=args.density,
        max_active=args.max_active,
        seed=args.seed,
        size_min=args.size_min,
        size_max=args.size_max,
        angle_min=args.angle_min,
        angle_max=args.angle_max,
        type_speed=args.type_speed,
        post_hold=args.post_hold,
        fonts=fonts,
    )

    print(f"Generated {len(overlays)} overlays")
    for o in overlays:
        print(f"  [{o['start_time']:.1f}s-{o['end_time']:.1f}s] {o['text']} "
              f"@ ({o['x']:.0f},{o['y']:.0f}) size={o['font_size']} angle={o['angle']:.1f}°")

    # Process video
    print()
    process_video(args.input, args.output, overlays, fonts)
    print(f"\nDone! Output: {args.output}")

if __name__ == "__main__":
    main()
