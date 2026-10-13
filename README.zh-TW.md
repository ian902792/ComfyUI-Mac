# ComfyUI-Mac

[English](README.md) · **繁體中文**

在 Apple Silicon Mac 上本機執行 [ComfyUI](https://github.com/comfyanonymous/ComfyUI) 的原生啟動器與安裝腳本。

- **ComfyUI.app**：約 11 MB 的 Tauri 2（Rust + 系統 WKWebView）外殼。負責啟動伺服器、顯示 log 視窗、從選單重啟與更新 ComfyUI，關閉時一併停掉伺服器。
- **腳本**：一行指令安裝（uv + Python 3.12 + torch/MPS + ComfyUI-Manager + ComfyUI-GGUF），模型下載可續傳。
- **實測數據**：MacBook Pro M5 Pro / 64 GB 的速度、記憶體與 MPS 相容性（見 [BENCHMARK.md](BENCHMARK.md)）。

所有東西都在同一個資料夾（預設 `~/GitHub/ComfyUI`），刪掉就乾淨。移除方式見 [UNINSTALL.md](UNINSTALL.md)。

## 快速開始

需要 Apple Silicon、macOS 13+、[uv](https://docs.astral.sh/uv/)、git。

```bash
git clone https://github.com/ian902792/ComfyUI-Mac && cd ComfyUI-Mac
bash scripts/install.sh                  # ComfyUI + venv + torch + 自訂節點
bash scripts/download_models.sh zimage   # 先下最小的一組（20 GB）；不帶參數則下載全部（約 55 GB）
bash scripts/start.sh                    # 然後開 http://127.0.0.1:8188
```

想用 App 而不是 `start.sh`，先編譯一次（見下節）。用環境變數 `COMFYUI_DIR` 可改安裝路徑。

在 ComfyUI 裡：左側 **Templates** → 選範本 → 改 prompt → **Queue**（⌘↵）。第一張會慢 2–3 倍（在編譯 Metal kernel），看第二張的時間才準。

## ComfyUI.app

開 App → 啟動畫面 → 伺服器就緒後自動切到 ComfyUI 介面（獨立視窗）。同時 `http://127.0.0.1:8188` 在任何瀏覽器都能開。關視窗或 ⌘Q 會停止伺服器；App 被強制結束時，watchdog 也會把伺服器收掉。

| 選單 | 做什麼 |
|---|---|
| Server ▸ Restart Server (⌘R) | 改了節點、模型或參數後重啟 |
| Server ▸ Show Log Window (⌘L) | 看 python 輸出（同 `user/app.log`） |
| Server ▸ Edit Startup Args… | 打開 `user/app_args.txt`（一行一個旗標），存檔後 ⌘R |
| Server ▸ Copy URL | 複製 `http://127.0.0.1:8188` |
| Update ▸ Update ComfyUI | 停伺服器 → `git pull` ComfyUI 與自訂節點 → 重裝依賴 → 重啟 |

若啟動時 8188 已經有 ComfyUI 在跑（例如你用 `start.sh` 開的），App 會直接接上，標題顯示 **(external server)**；離開時不會關它，Restart / Update 也會停用。

### 編譯 App

```bash
cd app/src-tauri && env -u MACOSX_DEPLOYMENT_TARGET cargo build --release
cd .. && pnpm dlx @tauri-apps/cli@2 bundle
rm -rf /Applications/ComfyUI.app && cp -R src-tauri/target/release/bundle/macos/ComfyUI.app /Applications/
```

> 不要用 `tauri build`。Tauri CLI 會設 `MACOSX_DEPLOYMENT_TARGET`，在 rustc 1.96 + macOS 27 上會讓所有 proc-macro 報 `can't find crate`（任何值都會，2026-09-18 實測）。先用純 cargo 編譯，再讓 `tauri bundle` 只負責打包。
>
> 若 `bundle` 卡住：中止它，把 `target/release/comfyui-mac` 複製到 `ComfyUI.app/Contents/MacOS/`，再執行 `codesign --force --deep -s - ComfyUI.app`。

App 是 ad-hoc 簽章，只在編譯它的這台 Mac 上能正常開。

## 模型

`scripts/download_models.sh [組名 ...]` 會下載到 `models/`，可續傳，已完整的檔會跳過。

| 組 | 模型 | 大小 | 適合 | 授權 |
|---|---|---|---|---|
| `zimage` | **Z-Image-Turbo** 6B，8 步 | 20 GB | 快速草稿、入門；唯一可商用 | Apache-2.0 |
| `flux` | **FLUX.1 Krea dev** 12B，GGUF Q8_0 | 18 GB | 真人皮膚質感最自然 | 非商用 |
| `qwen` | **Qwen Image 2.1** 7B，int8_convrot + Turbo LoRA | 18 GB | 文字渲染、提示詞理解準，原生 2K，生圖與編輯同一個模型 | Qwen Research（非商用） |
| `noct`（選用） | **Noct-Q V4**，Qwen Image 2.1 的寫實 merge | 7 GB | 可直接取代 Qwen 2.1 的 diffusion model | 見模型頁 |

Qwen Image 2.1 檔案（需要 ComfyUI ≥ 0.37）：

| 檔案 | 資料夾 | 大小 |
|---|---|---|
| `qwen_image_2.1_int8_convrot.safetensors` | `diffusion_models` | 7.3 GB |
| `qwen3vl_8b_int8_convrot.safetensors` | `text_encoders` | 9.4 GB |
| `qwen_image_2.1_vae_bf16.safetensors` | `vae` | 0.7 GB |
| `qwen_image_2.1_turbo_lora_avg_rank_178_bf16.safetensors` | `loras` | 0.9 GB |

Qwen 2.1 使用重點：
- 快速：掛 Turbo LoRA、8 步、cfg 1。高品質：25 步、cfg 3 並填負面提示詞（M5 Pro 上 1024×1536 約 7 分鐘）。
- 用完整句子描述，不要用標籤式 prompt。尺寸要是 32 的倍數，原生上限約 2048。
- 多圖編輯用 `<image1>`、`<image2>`；換背景實測可用，換角度 / 姿勢尚未測。
- 舊的 Qwen-Image / Qwen-Image-Edit LoRA（Lightning、多角度）**不能用在 2.1**，請用 2.1 的 Turbo LoRA。（只比對了 tensor 形狀，沒有實際載入測試。）
- 完整說明與工作流程：[amenderstudio.com/notes/local-image-guide](https://amenderstudio.com/notes/local-image-guide/)。

## 速度與記憶體

MacBook Pro M5 Pro（20 核 GPU）、64 GB、ComfyUI 0.39、`--use-pytorch-cross-attention`。數字為第二張起的穩定速度（第一張含載入模型）。

| 模型 | 設定 | 時間 |
|---|---|---|
| Z-Image-Turbo | 1344×768，8 步 | **22 s**（首張約 50 s） |
| FLUX.1 Krea GGUF Q8 | 1344×768，20 步 | **85 s**（首張約 110 s） |
| Qwen Image 2.1 + Turbo LoRA | 1024²，8 步 | **48–50 s**（首張約 77 s） |
| Qwen Image 2.1 | 1024²，25 步，cfg 1 | 173 s |
| Qwen Image 2.1 | 1024×1536，25 步，cfg 3 + 負面提示詞 | 426 s（含載入） |
| Qwen Image 2.1 編輯 + Turbo | 約 1 MP，8 步 | 取樣約 53 s（首次 111 s，含載入） |

Z-Image 整機峰值約 51 GB、Flux 約 46 GB（都含約 17.5 GB 的日常基準用量）。跑 Qwen 前請先關掉吃記憶體的 app。換模型要重新載入，所以同一個模型的工作集中做完再換。

舊版 Qwen-Image 1.x / Edit-2511 的實測留在 [BENCHMARK.md](BENCHMARK.md) 供參考，下載腳本已不再包含這些模型。

## 設定

啟動旗標寫進 `user/app_args.txt`（或接在 `start.sh` 後面）。

| 旗標 | 結論 |
|---|---|
| `--use-pytorch-cross-attention` | **預設開啟**。用 PyTorch 融合 SDPA 取代 ComfyUI 在 MPS 的 sub-quadratic 後備實作：Z-Image −24%、Flux −13%、Qwen-Edit −22%，數學等價，輸出相同 |
| `--fp16-vae` | 只有出黑圖 / 灰圖時才用；M5 原生支援 bf16 |
| `--preview-method auto` | 生成中即時預覽，但慢約 5–10% |
| `--output-directory <路徑>` | 輸出改到別處 |
| `--fast`、`--fp8*`、`--highvram`、`--gpu-only`、`--cache-lru N` | 不建議：有畫質風險、MPS 沒有 fp8 dtype，或統一記憶體下沒有收益 |
| `--async-offload` | MPS 上未測 |

SageAttention 在 Metal 沒有 kernel，不用試。

## MPS 相容性

| 檔案精度 | 能跑？ | 備註 |
|---|---|---|
| bf16 / fp16 | ✔ | 原生 |
| fp8_e4m3fn（一般） | ✔ | 自動升為 bf16 |
| **fp8_scaled / fp8mixed** | ✘ | `Undefined type Float8_e4m3fn`；改用 GGUF、int8 或 bf16 |
| int8_convrot | ✔ | ComfyUI 官方 int8。MPS 上會停用 int8 matmul，逐層反量化成 bf16 計算（約為純 bf16 層的 2 倍時間）。不要直接呼叫 comfy_kitchen 的 `int8_linear`：MPS 沒有實作 `aten::_int_mm` |
| GGUF Q8_0 | ✔ | 需要 `custom_nodes/ComfyUI-GGUF` |
| 文字編碼器 `*_fp8_scaled`（如 T5-XXL） | ✔ | 例外，能跑 |

## 目錄結構

```
ComfyUI-Mac/               本 repo
├── app/                   ComfyUI.app 原始碼（Tauri 2 / Rust）
├── scripts/               install.sh · download_models.sh · start.sh · bench.py
├── BENCHMARK.md           詳細實測
└── UNINSTALL.md

~/GitHub/ComfyUI/          官方 ComfyUI 的乾淨 clone（git status 為空，可 git pull）
├── .python/  .venv/       自包含 Python 3.12 + torch（MPS）
├── custom_nodes/          ComfyUI-Manager、ComfyUI-GGUF
├── models/  input/  output/
└── user/                  app_args.txt、app.log
```

手動更新：`cd ~/GitHub/ComfyUI && git pull && uv pip install --python .venv/bin/python -r requirements.txt`，或用 App 的 Update 選單。

## 跟官方 ComfyUI Desktop 的差別

| | 這套 | [ComfyUI Desktop](https://www.comfy.org/download) |
|---|---|---|
| 核心 | 官方 git clone，`git pull` 就是最新 | 同一套 ComfyUI，跟隨穩定版，落後幾天到幾週 |
| 外殼 | Tauri + 系統 WebView，約 11 MB | Electron，約 200–300 MB，多吃 200–400 MB RAM |
| 檔案位置 | 一個資料夾，刪掉即移除 | App + Application Support + base path 三處 |
| 更新 | 手動，自己決定時機 | 自動提示，一鍵升級 |
| 簽章 | ad-hoc，只在這台 Mac | Apple 公證 |
| 額外功能 | log 視窗、參數編輯、重啟 | 設定 UI、內建終端機、崩潰回報、安裝精靈 |

兩者可並存（Desktop 預設 port 8000）；把 Desktop 的 model path 指到 `~/GitHub/ComfyUI/models` 即可共用模型。

## 常見問題

| 症狀 | 處理 |
|---|---|
| 比預期慢 3 倍以上 | 第一張在編譯 kernel 屬正常；用活動監視器看記憶體壓力，關掉吃 GPU 的 app |
| 黑圖 / 灰圖 | `app_args.txt` 加 `--fp16-vae`，⌘R |
| `MPS backend out of memory` | 降解析度；或 `PYTORCH_MPS_HIGH_WATERMARK_RATIO=0.0 scripts/start.sh` |
| Template 找不到模型 | 檔名與子資料夾要跟 template 期待的一致（見「模型」）；按 `R` 重新整理 |
| 紅色（缺）節點 | Manager → Install Missing Custom Nodes |
| 下載中斷 | 重跑 `download_models.sh`，會自動續傳 |
| App 卡在啟動畫面 | ⌘L 看 log；多半是 python 例外，或 `app_args.txt` 打錯旗標 |

## 授權

本 repo 的程式碼採 [MIT](LICENSE)。ComfyUI 是 GPL-3.0，安裝時才 clone，不包含在本 repo。各模型有自己的授權（見「模型」）：Z-Image 為 Apache-2.0，FLUX.1 Krea dev 與 Qwen Image 2.1 為非商用。
