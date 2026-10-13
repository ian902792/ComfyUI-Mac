# ComfyUI-Mac

**English** · [繁體中文](README.zh-TW.md)

A small native macOS launcher plus setup scripts for running [ComfyUI](https://github.com/comfyanonymous/ComfyUI) locally on Apple Silicon.

- **ComfyUI.app** — an 11 MB Tauri 2 (Rust + system WKWebView) wrapper. It starts the server, shows a log window, restarts and updates ComfyUI from the menu, and stops everything when you quit.
- **Scripts** — one-command install (uv + Python 3.12 + torch/MPS + ComfyUI-Manager + ComfyUI-GGUF) and resumable model downloads.
- **Measured numbers** — speed, memory and MPS compatibility on a MacBook Pro M5 Pro / 64 GB ([BENCHMARK.md](BENCHMARK.md), in Chinese).

Everything lives in one folder (`~/GitHub/ComfyUI` by default) and can be removed by deleting it. See [UNINSTALL.md](UNINSTALL.md).

## Quick start

Requires Apple Silicon, macOS 13+, [uv](https://docs.astral.sh/uv/), git.

```bash
git clone https://github.com/ian902792/ComfyUI-Mac && cd ComfyUI-Mac
bash scripts/install.sh              # ComfyUI + venv + torch + custom nodes
bash scripts/download_models.sh zimage   # start small (20 GB); omit the argument for all groups (~55 GB)
bash scripts/start.sh                # then open http://127.0.0.1:8188
```

To use the app instead of `start.sh`, build it once (next section). Set `COMFYUI_DIR` to install somewhere other than `~/GitHub/ComfyUI`.

In ComfyUI: **Templates** (left sidebar) → pick a template → edit the prompt → **Queue** (⌘↵). The first image is 2–3× slower while Metal kernels compile; judge speed by the second one.

## ComfyUI.app

Open the app → splash screen → the ComfyUI UI appears in its own window once the server is ready. `http://127.0.0.1:8188` also works in any browser. Closing the window or quitting stops the server; a watchdog cleans it up even if the app is force-killed.

| Menu | Action |
|---|---|
| Server ▸ Restart Server (⌘R) | Restart after changing nodes, models or args |
| Server ▸ Show Log Window (⌘L) | Python output (same as `user/app.log`) |
| Server ▸ Edit Startup Args… | Opens `user/app_args.txt` (one flag per line), then ⌘R |
| Server ▸ Copy URL | Copies `http://127.0.0.1:8188` |
| Update ▸ Update ComfyUI | Stops the server, `git pull`s ComfyUI and custom nodes, reinstalls requirements, restarts |

If something is already listening on 8188 (e.g. you ran `start.sh`), the app attaches to it and shows **(external server)**; it won't stop it on exit, and Restart/Update are disabled.

### Build the app

```bash
cd app/src-tauri && env -u MACOSX_DEPLOYMENT_TARGET cargo build --release
cd .. && pnpm dlx @tauri-apps/cli@2 bundle
rm -rf /Applications/ComfyUI.app && cp -R src-tauri/target/release/bundle/macos/ComfyUI.app /Applications/
```

> Don't use `tauri build`. The Tauri CLI sets `MACOSX_DEPLOYMENT_TARGET`, which on rustc 1.96 + macOS 27 makes every proc-macro fail with `can't find crate` (any value, tested 2026-09-18). Compile with plain cargo first, then let `tauri bundle` only package.
>
> If `bundle` hangs, kill it and copy `target/release/comfyui-mac` into `ComfyUI.app/Contents/MacOS/`, then `codesign --force --deep -s - ComfyUI.app`.

The app is ad-hoc signed, so it only runs cleanly on the Mac that built it.

## Models

`scripts/download_models.sh [group ...]` downloads into `models/`. It is resumable and skips complete files.

| Group | Model | Size | Best for | License |
|---|---|---|---|---|
| `zimage` | **Z-Image-Turbo** 6B, 8 steps | 20 GB | Fast drafts, first run; Apache-2.0 so usable commercially | Apache-2.0 |
| `flux` | **FLUX.1 Krea dev** 12B, GGUF Q8_0 | 18 GB | Most natural realistic skin and photo look | Non-commercial |
| `qwen` | **Qwen Image 2.1** 7B, int8_convrot + Turbo LoRA | 18 GB | Accurate text rendering, prompt following, native 2K, text-to-image **and** editing in one model | Qwen Research (non-commercial) |
| `noct` (opt-in) | **Noct-Q V4**, a realistic merge of Qwen Image 2.1 | 7 GB | Drop-in replacement for the Qwen 2.1 diffusion model | see model card |

Qwen Image 2.1 files (ComfyUI ≥ 0.37 required):

| File | Folder | Size |
|---|---|---|
| `qwen_image_2.1_int8_convrot.safetensors` | `diffusion_models` | 7.3 GB |
| `qwen3vl_8b_int8_convrot.safetensors` | `text_encoders` | 9.4 GB |
| `qwen_image_2.1_vae_bf16.safetensors` | `vae` | 0.7 GB |
| `qwen_image_2.1_turbo_lora_avg_rank_178_bf16.safetensors` | `loras` | 0.9 GB |

Tips for Qwen 2.1:
- Fast preset: Turbo LoRA, 8 steps, cfg 1. Quality preset: 25 steps, cfg 3 with a negative prompt (≈7 min at 1024×1536 on an M5 Pro).
- Write full sentences, not tag lists. Sizes must be multiples of 32; native maximum ≈ 2048.
- Multi-image editing uses `<image1>`, `<image2>`; background swap works well. Angle/pose edits are untested.
- Old Qwen-Image / Qwen-Image-Edit LoRAs (Lightning, multi-angle) **do not work** with 2.1; use the 2.1 Turbo LoRA. (Checked by comparing tensor shapes, not by a load test.)
- Upstream details and workflows: [amenderstudio.com/notes/local-image-guide](https://amenderstudio.com/notes/local-image-guide/).

## Speed and memory

MacBook Pro M5 Pro (20-core GPU), 64 GB, ComfyUI 0.39, `--use-pytorch-cross-attention`. Steady-state seconds per image after the first (which includes model load).

| Model | Setup | Time |
|---|---|---|
| Z-Image-Turbo | 1344×768, 8 steps | **22 s** (first ≈ 50 s) |
| FLUX.1 Krea GGUF Q8 | 1344×768, 20 steps | **85 s** (first ≈ 110 s) |
| Qwen Image 2.1 + Turbo LoRA | 1024², 8 steps | **48–50 s** (first ≈ 77 s) |
| Qwen Image 2.1 | 1024², 25 steps, cfg 1 | 173 s |
| Qwen Image 2.1 | 1024×1536, 25 steps, cfg 3 + negative | 426 s (incl. load) |
| Qwen Image 2.1 edit + Turbo | ≈ 1 MP, 8 steps | ≈ 53 s sampling (111 s first, incl. load) |

Z-Image peaks around 51 GB system memory and Flux around 46 GB (both include ≈ 17.5 GB of baseline use). Close other heavy apps before running Qwen. Switching models costs a reload, so batch your work per model.

The older Qwen-Image 1.x / Edit-2511 measurements are kept in [BENCHMARK.md](BENCHMARK.md) for reference; those models are no longer part of the download script.

## Settings

Startup flags go in `user/app_args.txt` (or after `start.sh`).

| Flag | Verdict |
|---|---|
| `--use-pytorch-cross-attention` | **On by default.** Uses PyTorch's fused SDPA instead of ComfyUI's sub-quadratic fallback on MPS: Z-Image −24%, Flux −13%, Qwen-Edit −22%, same math and output |
| `--fp16-vae` | Only if you get black or gray images; M5 has native bf16 |
| `--preview-method auto` | Live preview, but ≈ 5–10% slower |
| `--output-directory <path>` | Send output elsewhere |
| `--fast`, `--fp8*`, `--highvram`, `--gpu-only`, `--cache-lru N` | Not recommended: quality risk, no fp8 dtype on MPS, or no gain with unified memory |
| `--async-offload` | Untested on MPS |

SageAttention has no Metal kernel; don't bother.

## MPS compatibility

| File precision | Works? | Notes |
|---|---|---|
| bf16 / fp16 | ✔ | Native |
| fp8_e4m3fn (plain) | ✔ | Upcast to bf16 automatically |
| **fp8_scaled / fp8mixed** | ✘ | `Undefined type Float8_e4m3fn`; use GGUF, int8 or bf16 |
| int8_convrot | ✔ | Official ComfyUI int8. On MPS int8 matmul is disabled and each layer is dequantized to bf16 (≈ 2× a pure bf16 layer). Don't call comfy_kitchen's `int8_linear` directly: `aten::_int_mm` is not implemented on MPS |
| GGUF Q8_0 | ✔ | Needs `custom_nodes/ComfyUI-GGUF` |
| Text encoders `*_fp8_scaled` (e.g. T5-XXL) | ✔ | The exception |

## Layout

```
ComfyUI-Mac/               this repo
├── app/                   ComfyUI.app source (Tauri 2 / Rust)
├── scripts/               install.sh · download_models.sh · start.sh · bench.py
├── BENCHMARK.md           detailed measurements (Chinese)
└── UNINSTALL.md

~/GitHub/ComfyUI/          clean upstream clone (git status stays empty, git pull to update)
├── .python/  .venv/       self-contained Python 3.12 + torch (MPS)
├── custom_nodes/          ComfyUI-Manager, ComfyUI-GGUF
├── models/  input/  output/
└── user/                  app_args.txt, app.log
```

Update manually with `cd ~/GitHub/ComfyUI && git pull && uv pip install --python .venv/bin/python -r requirements.txt`, or use the app's Update menu.

## vs. official ComfyUI Desktop

| | This setup | [ComfyUI Desktop](https://www.comfy.org/download) |
|---|---|---|
| Core | Upstream git clone; `git pull` gives you the latest | Same ComfyUI, follows stable releases a few days to weeks behind |
| Shell | Tauri + system WebView, ≈ 11 MB | Electron, ≈ 200–300 MB, 200–400 MB more RAM |
| Files | One folder, delete to uninstall | App + Application Support + base path |
| Updates | Manual, you choose when | Automatic prompts, one click |
| Signing | Ad-hoc, this Mac only | Notarized |
| Extras | Log window, args editor, restart | Settings UI, built-in terminal, crash reports, installer |

They coexist (Desktop uses port 8000); point Desktop's model path at `~/GitHub/ComfyUI/models` to share models.

## Troubleshooting

| Symptom | Fix |
|---|---|
| 3× slower than expected | First image compiles kernels; check memory pressure in Activity Monitor and close GPU-heavy apps |
| Black or gray images | Add `--fp16-vae` to `app_args.txt`, ⌘R |
| `MPS backend out of memory` | Lower the resolution, or `PYTORCH_MPS_HIGH_WATERMARK_RATIO=0.0 scripts/start.sh` |
| Template can't find a model | File name and subfolder must match what the template expects (see Models); press `R` to refresh |
| Red (missing) nodes | Manager → Install Missing Custom Nodes |
| Interrupted download | Re-run `download_models.sh`; it resumes |
| App stuck on splash | ⌘L for the log; usually a Python exception or a typo in `app_args.txt` |

## License

[MIT](LICENSE) for the code in this repo. ComfyUI is GPL-3.0 and is cloned at install time, not included here. Each model has its own license (see Models); Z-Image is Apache-2.0, while FLUX.1 Krea dev and Qwen Image 2.1 are non-commercial.
