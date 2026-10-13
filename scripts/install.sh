#!/usr/bin/env bash
# 從零重建 ~/GitHub/ComfyUI（自包含：Python、venv、pip 快取、custom_nodes 都在裡面）。可重複執行。
#   bash scripts/install.sh            # 只裝程式
#   bash scripts/install.sh --models   # 順便下載全部模型（約 90 GB）
set -euo pipefail
D="${COMFYUI_DIR:-$HOME/GitHub/ComfyUI}"
command -v uv >/dev/null || { echo "需要 uv：curl -LsSf https://astral.sh/uv/install.sh | sh"; exit 1; }
[ -d "$D/.git" ] || git clone https://github.com/comfyanonymous/ComfyUI "$D"
cd "$D"
export UV_PYTHON_INSTALL_DIR="$D/.python" UV_CACHE_DIR="$D/.uv-cache"
[ -x .venv/bin/python ] || uv venv --python 3.12 .venv
uv pip install --python .venv/bin/python torch torchvision torchaudio
uv pip install --python .venv/bin/python -r requirements.txt
for repo in Comfy-Org/ComfyUI-Manager city96/ComfyUI-GGUF; do
  n=${repo#*/}; [ -d "custom_nodes/$n" ] || git clone --depth 1 "https://github.com/$repo" "custom_nodes/$n"
  [ -f "custom_nodes/$n/requirements.txt" ] && uv pip install --python .venv/bin/python -r "custom_nodes/$n/requirements.txt"
done
mkdir -p user; [ -f user/app_args.txt ] || printf '# ComfyUI.app 額外啟動參數，一行一個。改完用選單 Server ▸ Restart Server。\n--use-pytorch-cross-attention\n# --fp16-vae\n# --preview-method auto\n' > user/app_args.txt
rm -rf .uv-cache
.venv/bin/python -c "import torch;print('torch',torch.__version__,'mps',torch.backends.mps.is_available())"
[ "${1:-}" = "--models" ] && bash "$(dirname "$0")/download_models.sh"
echo "done: $D"
