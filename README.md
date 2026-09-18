# ComfyUI-Mac — 本機 AI 生圖環境（MacBook Pro M5 Pro / 64 GB）

建於 2026-09-18。目的：在本機用 ComfyUI 產「之後要餵給雲端 MiniMax H3 影片流程」的靜態素材
（角色設計、參考圖集、場景關鍵幀）。影片生成不在本機做，去 RunPod（見 `../RunpodGenVideo/`）。

```
ComfyUI-Mac/                  ← 這個 repo 資料夾：我們自己的東西，有版控
├── README.md                 本檔
├── BENCHMARK.md              實測速度 / 記憶體 / MPS 相容表
├── UNINSTALL.md
├── docs/local-image-guide.html   新手教學（模型選擇、時間、哪些素材接回影片）
├── scripts/
│   ├── install.sh            從零重建 ~/ComfyUI（clone + venv + torch + custom_nodes）
│   ├── download_models.sh    下載模型（可續傳、可單組）
│   ├── start.sh              終端機啟動（平常用 App）
│   └── bench.py              測速（速度 + 記憶體）
└── app/                      ComfyUI.app 原始碼（Tauri 2 / Rust）

~/ComfyUI/                    ← 官方 ComfyUI 的乾淨 clone（git status 為空，可 git pull）
├── .python/ .venv/           自包含 Python 3.12 + torch(MPS)          （官方 .gitignore 已忽略）
├── custom_nodes/             ComfyUI-Manager、ComfyUI-GGUF            （同上）
├── models/                   86 GB                                     （同上）
├── user/app_args.txt         App 額外啟動參數                          （同上）
└── user/app.log              App 啟動的伺服器日誌                      （同上）

/Applications/ComfyUI.app     ← 用這個開
```

## 0. ComfyUI.app（平常就用這個）

開 App → 啟動畫面 → 伺服器就緒後自動切到 ComfyUI 介面（獨立視窗）。同時 **http://127.0.0.1:8188 在任何瀏覽器都能開**。
關閉視窗或 Cmd+Q → 伺服器一起停；App 被強制結束（crash / kill -9）也會由 watchdog 把伺服器收掉，不會留孤兒程序。

| 選單 | 做什麼 |
|---|---|
| Server ▸ Restart Server (⌘R) | 改了 custom_nodes / 模型 / 參數後重啟 |
| Server ▸ Show Log Window (⌘L) | 看 python 輸出（同 `~/ComfyUI/user/app.log`） |
| Server ▸ Edit Startup Args… | 打開 `user/app_args.txt`（一行一個旗標，如 `--fp16-vae`），存檔後 ⌘R |
| Server ▸ Copy URL | 複製 http://127.0.0.1:8188 |
| Update ▸ Update ComfyUI (git pull)… | 停伺服器 → `git pull` ComfyUI 與 custom_nodes → `uv pip install -r requirements.txt` → 重啟；過程在 log 視窗 |

若啟動時 8188 已經有 ComfyUI 在跑（例如你用 `scripts/start.sh` 開的），App 會直接接上，標題顯示 **(external server)**，離開時不會關它；Restart / Update 在這個模式下停用。
環境變數 `COMFYUI_DIR` 可改安裝路徑（預設 `~/ComfyUI`）。

重新編譯（改了 `app/` 之後）：
```bash
cd ComfyUI-Mac/app/src-tauri && env -u MACOSX_DEPLOYMENT_TARGET cargo build --release
cd .. && pnpm dlx @tauri-apps/cli@2 bundle && rm -rf /Applications/ComfyUI.app && cp -R src-tauri/target/release/bundle/macos/ComfyUI.app /Applications/
```
> 為什麼不用 `tauri build`：Tauri CLI 會設 `MACOSX_DEPLOYMENT_TARGET`，在 rustc 1.96 + macOS 27 上會讓所有 proc-macro 報 `can't find crate`（2026-09-18 實測，任何值都會）。先用純 cargo 編譯，再用 `tauri bundle` 只打包不重編即可。

## 1. 環境

| 項目 | 內容 |
|---|---|
| ComfyUI | v0.36.0，`git clone` 官方 repo；用 App 的 Update 選單或 `git -C ~/ComfyUI pull` 更新 |
| 自訂節點 | `~/ComfyUI/custom_nodes/ComfyUI-Manager`（在 UI 內找/裝缺的節點與模型）<br>`custom_nodes/ComfyUI-GGUF`（載入 .gguf 模型，Flux Krea 用） |
| Python | 3.12.13，uv 管理，裝在 **`.python/`**（不是系統 Python 3.14） |
| 虛擬環境 | **`.venv/`**；torch 2.14（MPS 可用）、ComfyUI requirements |
| pip 快取 | `.uv-cache/`（已清；安裝/更新時 uv 會再建，可隨時刪） |
| 模型 | **`models/`**（見 §2） |
| 輸出 / 輸入 | `output/` / `input/` |

執行環境全部在 `~/ComfyUI`（沒有 Homebrew 套件、`~/.cache/huggingface`、`~/Library` 殘留）；移除見 `UNINSTALL.md`。
從零重建：`bash scripts/install.sh --models`。

## 2. 模型（`models/`，共約 90 GB）

| 組 | 檔案 | 子資料夾 | 用途 |
|---|---|---|---|
| **zimage** | `z_image_turbo_bf16.safetensors` 12.3 GB<br>`qwen_3_4b.safetensors` 8.0 GB<br>`ae.safetensors` 0.3 GB | diffusion_models<br>text_encoders<br>vae | Z-Image-Turbo：8 步出圖、最快，草稿與入門（Apache-2.0） |
| **flux** | `flux1-krea-dev-Q8_0.gguf` 12.7 GB（放 **unet/**，用 UnetLoaderGGUF）<br>`clip_l.safetensors` + `t5xxl_fp8_e4m3fn_scaled.safetensors` 5.4 GB<br>（vae 共用 `ae.safetensors`） | 同上 | FLUX.1 Krea dev：真人質感最自然，角色主圖定稿（**非商用**授權） |
| **qwen** | `qwen_image_fp8_e4m3fn.safetensors` 20.4 GB<br>`qwen_2.5_vl_7b_fp8_scaled.safetensors` 9.4 GB<br>`qwen_image_vae.safetensors` 0.3 GB<br>`Qwen-Image-Lightning-8steps-V2.0.safetensors` 1.7 GB | 同上 + loras | Qwen-Image：提示詞理解/構圖/文字最準，場景關鍵幀（Apache-2.0） |
| **edit** | `qwen_image_edit_2511_int8_convrot.safetensors` 20.5 GB<br>`Qwen-Image-Edit-2511-Lightning-4steps-V1.0-bf16.safetensors` 0.85 GB<br>`Qwen-Edit-2509-Multiple-angles.safetensors` 0.24 GB（一鍵多角度）<br>（text encoder / vae 與 qwen 共用） | 同上 | Qwen-Image-Edit：多圖輸入、換角度/表情/服裝但保持同一人 |

下載：`bash scripts/download_models.sh`（全部）或 `bash scripts/download_models.sh zimage`（單組）。可中斷續傳，已完整的檔會跳過。

## 3. 使用

開 `/Applications/ComfyUI.app`（或終端機 `scripts/start.sh`，Ctrl+C 結束）。
1. 左側 **Templates** → Image → 選 `Z-Image Turbo` / `Flux Krea` / `Qwen Image` / `Qwen Image Edit`，模型已在本機不用再下。
2. 改 prompt、解析度、seed → **Queue**（或 Cmd+Enter）。
3. 結果在 Save Image 節點與 `output/`。
4. 第一張會慢 2–3 倍（編譯 Metal kernel），看第二張的時間才準。

實用旗標（寫進 `~/ComfyUI/user/app_args.txt`，或接在 `start.sh` 後）：
- `--fp16-vae`：出黑圖/灰圖時用
- `--preview-method auto`：生成中即時預覽
- `--output-directory ~/AIVideo/out`：輸出改到別處

更新：App 選單 Update ▸ Update ComfyUI，或 `cd ~/ComfyUI && git pull && uv pip install --python .venv/bin/python -r requirements.txt`

## 4. 學習計畫（每晚 1 小時，四週）

目標：四週後能穩定產出「同一個角色、不同場景」的 1344×768 素材，直接進雲端影片流程。

### 第 1 週：會用 + 有感覺（Z-Image-Turbo）
- [ ] D1 啟動、跑 Z-Image template 出第一張；記下第二張秒數
- [ ] D2 同一 prompt 換 5 個 seed，看什麼變、什麼不變；學會固定 seed
- [ ] D3 只改一個詞（光線 / 鏡頭 / 年代），各抽 3 張，體會 prompt 的槓桿在哪
- [ ] D4 解析度改 1344×768（16:9），試 9:16；理解為什麼要 32 的倍數
- [ ] D5 寫第一份「角色聖經」（80–150 字、3 個不尋常特徵），抽 20 張選 1 張當主圖
- [ ] 週末 對照 `BENCHMARK.md`，看自己的機器跑出來有沒有差很多

### 第 2 週：質感（FLUX.1 Krea）
- [ ] D1 同一份角色聖經跑 Flux Krea，跟 Z-Image 並排比，決定角色「住」哪個模型
- [ ] D2 學 steps / guidance：20 vs 28 步、guidance 2.5 vs 3.5 vs 5
- [ ] D3 img2img：把 Z-Image 草稿丟給 Flux 重繪（denoise 0.4–0.6），學「草稿→定稿」
- [ ] D4 去 AI 味：加雀斑、亂髮、自然窗光、35mm，與負面詞比對
- [ ] D5 主參考圖定稿，檔名帶 seed，存到 `~/AIVideo/characters/<name>/`

### 第 3 週：一致性（Qwen-Image-Edit）
- [ ] D1 跑 Qwen Image Edit template：主圖 + 「same person, turn 45°」
- [ ] D2 做 3/4 側、側面、全身各 1 張；失敗就換說法，記下有效的指令句型
- [ ] D3 5 種表情、3 種光線
- [ ] D4 換 2 套服裝（保持臉）；做服裝設定圖
- [ ] D5 用 Qwen-Image 生「空場景」1344×768，再用 Edit 把角色放進去 → 第一張場景關鍵幀
- [ ] 週末 一致性測試：5 個場景並排看臉飄不飄；補參考圖

### 第 4 週：接回影片流程
- [ ] D1 做文章同款「左景右人」合成圖（Image Concatenate 節點）
- [ ] D2 為一個 60 秒小故事寫分鏡表 `shots.csv`（4 段 × 15 秒），每段做首幀
- [ ] D3 做首尾幀對（同場景兩張，姿勢不同）
- [ ] D4 整理資料夾命名（見 local-image-guide §7），全部 PNG、16:9
- [ ] D5 開 RunPod 5090 一次（照 `ai-girlfriend/IMPLEMENT.md`），把本週素材變成 60 秒影片
- [ ] 回顧：哪些素材真的被用到、哪些白做 → 修正下個月的做法

### 之後（選修）
- 訓練自己的角色 LoRA（RunPod 5090 上 30–60 分鐘，用第 3 週的 20–30 張圖）
- ControlNet / 姿勢控制、放大流程、ComfyUI API 批次

## 5. 實測速度與記憶體（本機）

完整紀錄與解讀見 **`BENCHMARK.md`**。摘要（1344×768，穩定速度，峰值為整機記憶體含 17.5 GB 基準）：

| 模型 | 步數 | 穩定 | 峰值記憶體 |
|---|---|---|---|
| Z-Image-Turbo | 8 | **29 s** | 51 GB |
| FLUX.1 Krea GGUF Q8 | 20 | **97 s** | 46 GB |
| Qwen-Image + Lightning | 8 | **33–56 s** | 61 GB ⚠ |
| Qwen-Image-Edit-2511 + Lightning | 4 | **68 s** | 61 GB ⚠ |

⚠ 跑 Qwen 系列前關掉其他吃記憶體的 app。重測：`~/ComfyUI/.venv/bin/python scripts/bench.py`。

## 6. 常見問題

| 症狀 | 處理 |
|---|---|
| 比預估慢 3 倍以上 | 第一張在編譯屬正常；活動監視器看記憶體壓力；關掉其他吃 GPU 的 app |
| 黑圖 / 灰圖 | `app_args.txt` 加 `--fp16-vae`，⌘R |
| `MPS backend out of memory` | 降解析度；或終端機 `PYTORCH_MPS_HIGH_WATERMARK_RATIO=0.0 scripts/start.sh` |
| Template 找不到模型 | 檔名要跟 template 期待的一致（§2），放錯子資料夾也找不到；改完按 R 重新整理 |
| 缺節點（紅色） | Manager → Install Missing Custom Nodes |
| 下載中斷 | 重跑 `bash scripts/download_models.sh`，自動續傳 |
| App 卡在啟動畫面 | ⌘L 看 log；多半是 python 例外，或 `app_args.txt` 打錯旗標 |
