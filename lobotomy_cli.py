#!/usr/bin/env python3
"""
Lobotomy Corporation Danmaku CLI
Usage:
  python3 lobotomy_cli.py -i input.mp4 -o output.mp4 --texts texts.json
  python3 lobotomy_cli.py -i input.mp4 -o output.mp4 --text "控制部" --color "#b43c3c"
"""

import argparse, json, os, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from danmaku_processor import (
    load_fonts, probe_video, make_overlays, process_video,
    load_text_specs, parse_color, FONT_DIR
)

def main():
    parser = argparse.ArgumentParser(
        description="Lobotomy Corporation Danmaku Generator",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s -i in.mp4 -o out.mp4 --texts my_texts.json
  %(prog)s -i in.mp4 -o out.mp4 --text "控制部" --color "#b43c3c"
  %(prog)s -i in.mp4 -o out.mp4 --text "WARNING" --text "ALERT" --density 0.5 --max-active 6
        """,
    )
    parser.add_argument("-i", "--input", required=True, help="Input video file")
    parser.add_argument("-o", "--output", default="output.mp4", help="Output video file")

    text_group = parser.add_argument_group("Text input (choose one)")
    text_group.add_argument("--texts", help="JSON file with text/color specs")
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
    elif args.text:
        color = parse_color(args.color)
        text_specs = [(t, color) for t in args.text]
    else:
        print("Error: Must specify --texts or --text", file=sys.stderr)
        sys.exit(1)

    if not text_specs:
        print("Error: No texts specified", file=sys.stderr)
        sys.exit(1)