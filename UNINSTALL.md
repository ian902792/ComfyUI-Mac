# 移除方式

```bash
rm -rf /Applications/ComfyUI.app   # App
rm -rf ~/ComfyUI                   # ComfyUI 程式、Python、venv、模型（86 GB）、輸出、設定
```
沒有其他殘留（未使用 Homebrew、`~/.cache/huggingface`、`~/Library`）。
開發用的 Rust/Node 工具鏈（`~/.cargo`、`~/.rustup`、pnpm 快取）不是本專案裝的，不動。
這個 repo 資料夾（`ComfyUI-Mac/`）只有原始碼與文件，留著或刪掉都行；`app/src-tauri/target/` 是編譯暫存（~1 GB），可隨時刪。
