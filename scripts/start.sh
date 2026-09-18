#!/usr/bin/env bash
# 終端機啟動 ComfyUI → http://127.0.0.1:8188（Ctrl+C 結束）。平常用 /Applications/ComfyUI.app 即可。
D="${COMFYUI_DIR:-$HOME/ComfyUI}"
cd "$D" && exec .venv/bin/python main.py --listen 127.0.0.1 --port 8188 "$@"
