# ComfyUI-Mac

- 執行環境在 `~/GitHub/ComfyUI`（官方 clone，保持 `git status` 乾淨；我們的檔案只放這個資料夾）。App 設定/日誌在 `~/GitHub/ComfyUI/user/`。
- 改了 `app/` 要重編：`env -u MACOSX_DEPLOYMENT_TARGET cargo build --release` → `pnpm dlx @tauri-apps/cli@2 bundle` → 複製到 /Applications（不要用 `tauri build`，原因見 README §0）。
- 改了模型清單：同步 `scripts/download_models.sh`、README §2、BENCHMARK.md、amenderstudio.com/notes/local-image-guide（模型選擇一節）。
