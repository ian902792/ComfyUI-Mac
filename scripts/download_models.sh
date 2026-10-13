#!/usr/bin/env bash
# Download models into $COMFYUI_DIR/models (default ~/GitHub/ComfyUI). 下載本機生圖用模型（可中斷續傳，重跑會跳過已完成檔）。
#   bash scripts/download_models.sh            # zimage + flux + qwen（約 55 GB）；noct 為選用，需明確指定
#   bash scripts/download_models.sh zimage     # 只下 Z-Image（平衡方案入門，20 GB）
#   bash scripts/download_models.sh flux qwen noct
set -uo pipefail
M="${COMFYUI_DIR:-$HOME/GitHub/ComfyUI}/models"; HF=https://huggingface.co
get(){ mkdir -p "$M/$1"; f="$M/$1/${3:-$(basename "$2")}"
  remote=$(curl -sIL "$2" | grep -i '^content-length' | tail -1 | tr -dc 0-9); local_sz=$(stat -f%z "$f" 2>/dev/null || echo 0)
  [ -n "$remote" ] && [ "$local_sz" = "$remote" ] && { echo "skip $(basename "$f") (complete)"; return; }
  echo "==> $(basename "$f") ($local_sz/$remote)"; curl -sSL --retry 8 --retry-all-errors -C - -o "$f" "$2" && echo "ok  $(basename "$f")"; }
want(){ [ $# -eq 0 ] && return 0; for w in "$@"; do [ "$w" = "$1" ] && return 0; done; return 1; }
sel=("$@"); has(){ [ ${#sel[@]} -eq 0 ] && [ "$1" != noct ] && return 0; for s in "${sel[@]}"; do [ "$s" = "$1" ] && return 0; done; return 1; }

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
if has qwen; then     # Qwen Image 2.1（7B 生成端）：文字渲染、提示詞理解、T2I 與編輯同一模型。Qwen Research License，非商用
  Q=$HF/Comfy-Org/Qwen-Image-2.1/resolve/main
  get diffusion_models $Q/diffusion_models/qwen_image_2.1_int8_convrot.safetensors                  #  7.3 GB（官方 template 用這個）
  get text_encoders    $Q/text_encoders/qwen3vl_8b_int8_convrot.safetensors                         #  9.4 GB
  get vae              $Q/vae/qwen_image_2.1_vae_bf16.safetensors                                   #  0.7 GB
  get loras            $Q/loras/qwen_image_2.1_turbo_lora_avg_rank_178_bf16.safetensors             #  0.9 GB（8 步加速；舊 Lightning LoRA 不相容）
fi
if has noct; then     # 選用：Noct-Q 寫實 merge（Qwen Image 2.1），可取代上面的 diffusion model
  get diffusion_models $HF/Noctaluna/Noct-Q-Uncensored-Qwen-Image-2.1/resolve/main/NoctQ_V4_int8_convrot.safetensors   # 7.3 GB
fi
echo; du -sh "$M"/{diffusion_models,text_encoders,vae,loras} 2>/dev/null
