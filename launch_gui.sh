#!/bin/bash
# Launch Lobotomy Danmaku GUI
cd "$(dirname "$0")"
source /tmp/unityextract/bin/activate 2>/dev/null
python3 lobotomy_gui.py "$@"
