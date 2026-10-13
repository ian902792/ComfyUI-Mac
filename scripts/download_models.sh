#!/usr/bin/env bash
# 下載本機生圖用模型到 $COMFYUI_DIR/models（預設 ~/GitHub/ComfyUI）（可中斷續傳，重跑會跳過已完成檔）。
#   bash scripts/download_models.sh            # 全部（約 90 GB）
#   bash scripts/download_models.sh zimage     # 只下 Z-Image（平衡方案入門，20 GB）
#   bash scripts/download_models.sh flux qwen edit
set -uo pipefail
M="${COMFYUI_DIR:-$HOME/GitHub/ComfyUI}/models"; HF=https://huggingface.co
get(){ mkdir -p "$M/$1"; f="$M/$1/${3:-$(basename "$2")}"
  remote=$(curl -sIL "$2" | grep -i '^content-length' | tail -1 | tr -dc 0-9); local_sz=$(stat -f%z "$f" 2>/dev/null || echo 0)
  [ -n "$remote" ] && [ "$local_sz" = "$remote" ] && { echo "skip $(basename "$f") (complete)"; return; }
  echo "==> $(basename "$f") ($local_sz/$remote)"; curl -sSL --retry 8 --retry-all-errors -C - -o "$f" "$2" && echo "ok  $(basename "$f")"; }
want(){ [ $# -eq 0 ] && return 0; for w in "$@"; do [ "$w" = "$1" ] && return 0; done; return 1; }
sel=("$@"); has(){ [ ${#sel[@]} -eq 0 ] && return 0; for s in "${sel[@]}"; do [ "$s" = "$1" ] && return 0; done; return 1; }

if has zimage; then   # Z-Image-Turbo 6B，8 步，Apache-2.0 —— 先玩這個
  get diffusion_models $HF/Comfy-Org/z_image_turbo/resolve/main/split_files/diffusion_models/z_image_turbo_bf16.safetensors   # 12.3 GB
  get text_encoders    $HF/Comfy-Org/z_image_turbo/resolve/main/split_files/text_encoders/qwen_3_4b.safetensors               #  8.0 GB
  get vae              $HF/Comfy-Org/z_image_turbo/resolve/main/split_files/vae/ae.safetensors                                  #  0.3 GB (Flux VAE，Krea 共用)
fi
if has flux; then     # FLUX.1 Krea dev 12B，真人質感，非商用授權
  get unet             $HF/QuantStack/FLUX.1-Krea-dev-GGUF/resolve/main/flux1-krea-dev-Q8_0.gguf                                              # 12.7 GB (GGUF Q8；fp8_scaled 在 MPS 會報 Undefined type Float8，需 ComfyUI-GGUF 節點)
  get text_encoders    $HF/comfyanonymous/flux_text_encoders/resolve/main/clip_l.safetensors                                                     #  0.2 GB
  get text_encoders    $HF/comfyanonymous/flux_text_encoders/resolve/main/t5xxl_fp8_e4m3fn_scaled.safetensors                                    #  5.2 GB
  get vae              $HF/Comfy-Org/z_image_turbo/resolve/main/split_files/vae/ae.safetensors
fi
if has qwen; then     # Qwen-Image 20B，構圖/文字/指令最準，Apache-2.0
  get diffusion_models $HF/Comfy-Org/Qwen-Image_ComfyUI/resolve/main/split_files/diffusion_models/qwen_image_fp8_e4m3fn.safetensors            # 20.4 GB
  get text_encoders    $HF/Comfy-Org/Qwen-Image_ComfyUI/resolve/main/split_files/text_encoders/qwen_2.5_vl_7b_fp8_scaled.safetensors          #  9.4 GB
  get vae              $HF/Comfy-Org/Qwen-Image_ComfyUI/resolve/main/split_files/vae/qwen_image_vae.safetensors                                #  0.3 GB
  get loras            $HF/lightx2v/Qwen-Image-Lightning/resolve/main/Qwen-Image-Lightning-8steps-V2.0.safetensors                             #  1.7 GB (8 步加速)
fi
if has edit; then     # Qwen-Image-Edit-2511，多圖編輯保持同一人（角色參考圖集用）
  get diffusion_models $HF/Comfy-Org/Qwen-Image-Edit_ComfyUI/resolve/main/split_files/diffusion_models/qwen_image_edit_2511_int8_convrot.safetensors        # 20.5 GB (int8；fp8mixed 在 MPS 不能跑)
  get loras            $HF/lightx2v/Qwen-Image-Edit-2511-Lightning/resolve/main/Qwen-Image-Edit-2511-Lightning-4steps-V1.0-bf16.safetensors        # 0.85 GB (2511 官方 4 步加速，template 用這個)
  get loras            $HF/Comfy-Org/Qwen-Image-Edit_ComfyUI/resolve/main/split_files/loras/Qwen-Edit-2509-Multiple-angles.safetensors                # 0.24 GB (一鍵多角度：角色參考圖集用)
  # text encoder / vae 與 qwen 共用
fi
echo; du -sh "$M"/{diffusion_models,text_encoders,vae,loras} 2>/dev/null
