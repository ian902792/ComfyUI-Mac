# 本機實測紀錄 — MacBook Pro M5 Pro (20 核 GPU) / 64 GB (68.7 GB 可見) / macOS 27.0

ComfyUI 0.36.0 · torch 2.14.0 (MPS) · Python 3.12.13 · 解析度 1344×768 · 每個模型跑兩張，第二張為穩定速度。
工具：`.venv/bin/python bench.py [zimage|flux|qwen|edit]`，結果 `bench_results.txt`，圖 `output/bench/`。

## 2026-09-18 完整一輪（冷啟動，四個模型連跑）

系統閒置基準（ComfyUI 已啟動、瀏覽器等其他 app 開著）：**17.5 GB**。
「峰值記憶體」= 整台機器 total − available（ComfyUI /system_stats 在 MPS 上的算法），已含基準。

| 模型 | 檔案 / 精度 | 步數 | 首張 | 穩定 | 峰值記憶體（整機） | 淨增量 |
|---|---|---|---|---|---|---|
| Z-Image-Turbo | bf16 12.3 GB + Qwen3-4B 8 GB | 8, res_multistep | 42.7 s | **29.1 s** | 50.7 GB | ~33 GB |
| FLUX.1 Krea dev | GGUF Q8_0 12.7 GB + T5 fp8 5.2 GB | 20, euler | 104.9 s | **96.9 s** | 46.1 GB | ~29 GB |
| Qwen-Image + Lightning | fp8 20.4 GB + Qwen2.5-VL fp8 9.4 GB | 8, euler | 246.1 s | **32.8 s** | 61.3 GB | ~44 GB |
| Qwen-Image-Edit-2511 + Lightning | int8_convrot 20.5 GB（共用 VL 編碼器） | 4, euler | 108.6 s | **67.5 s** | 61.2 GB | ~44 GB |

前一輪（同日稍早，非冷啟動）：Z-Image 24.3 s、Flux 97.5 s、Qwen 55.9 s、Edit 64.1 s。
Qwen 8 步在 33–56 s 之間波動，取決於前一個模型是否還占著記憶體。

## 2026-09-18 加 `--use-pytorch-cross-attention`（PyTorch SDPA 取代 sub-quadratic attention）

| 模型 | 之前 | 之後 | 差 |
|---|---|---|---|
| Z-Image-Turbo 8 步 | 29.1 s | **22.2 s** | −24% |
| FLUX.1 Krea Q8 20 步 | 96.9 s | **84.5 s** | −13% |
| Qwen-Image 8 步 | 32.8–55.9 s | 44.1 s | 在原本波動範圍內 |
| Qwen-Edit-2511 4 步 | 67.5 s | **52.8 s** | −22% |

峰值記憶體不變（Qwen 系列仍 ~60 GB）。輸出是同一組 seed，畫面肉眼無差（SDPA 是數學等價的注意力實作，只是 kernel 不同）。
**已設為預設**（`~/ComfyUI/user/app_args.txt`）。

同時觀察到 `sysctl vm.swapusage` 顯示 28.8 GB swap 在用——跑 Qwen 系列時系統會把其他 app 換出去；不是錯誤，但代表 64 GB 對 20B 模型 + 日常 app 是剛剛好。

## 解讀

- **Z-Image 是日常草稿機**：30 秒一張、記憶體 50 GB 以下，可邊開瀏覽器邊跑。
- **Qwen-Image / Edit 會把 64 GB 機器推到 ~61 GB**：跑之前關掉吃記憶體的 app（瀏覽器多分頁、Docker、IDE），否則會 swap 變慢 3–10 倍。
- **Flux Krea 是四個裡最慢的（~100 s）** 但記憶體最省；20 步已夠，28 步約 2.3 分。
- 首張時間 = 從硬碟載入模型（Qwen 30 GB 要 3–4 分鐘）。同一模型連續出圖只付穩定時間；換模型就要再付一次載入。
  → 工作習慣：**同一個模型的工作集中做完再換模型**。
- 對照 RunPod RTX 5090：Z-Image 2–3 s、Flux ~10 s、Qwen ~20 s（快 10–30×）；本機贏在 $0 與零上傳。

## MPS 相容性（實測）

| 檔案精度 | 能跑？ | 備註 |
|---|---|---|
| bf16 / fp16 | ✔ | 原生 |
| fp8_e4m3fn（一般） | ✔ | ComfyUI 自動升 bf16 |
| **fp8_scaled / fp8mixed** | ✘ | `RuntimeError: Undefined type Float8_e4m3fn`；換 GGUF / int8 / bf16 |
| int8_convrot | ✔ | ComfyUI 官方 int8 |
| GGUF Q8_0 | ✔ | 需 `custom_nodes/ComfyUI-GGUF`，用 UnetLoaderGGUF |
| 文字編碼器 `*_fp8_scaled` | ✔ | 例外，能跑 |
