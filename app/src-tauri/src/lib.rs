mod server;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use server::{Server, State};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{AppHandle, Emitter, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

const READY_TIMEOUT: Duration = Duration::from_secs(180);

#[tauri::command]
fn get_log(st: tauri::State<State>) -> Vec<String> {
    st.lock().unwrap().log.iter().cloned().collect()
}

#[tauri::command]
fn get_status(st: tauri::State<State>) -> serde_json::Value {
    let s = st.lock().unwrap();
    serde_json::json!({ "dir": s.dir.display().to_string(), "external": s.external, "url": server::URL, "running": s.child.is_some() })
}

fn set_title(app: &AppHandle, suffix: &str) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_title(&format!("ComfyUI — 127.0.0.1:{}{}", server::PORT, suffix));
    }
}

fn show_splash(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.navigate(tauri::Url::parse("tauri://localhost/index.html").unwrap());
    }
    set_title(app, " (starting…)");
}

/// start() + background readiness poll → navigate main window to the ComfyUI UI.
fn launch(app: AppHandle) {
    let st: State = app.state::<State>().inner().clone();
    if let Err(msg) = server::start(&app, &st) {
        server::push_log(&app, &st, format!("[app] {msg}"));
        let _ = app.emit("server-failed", &msg);
        set_title(&app, " (failed)");
        return;
    }
    std::thread::spawn(move || {
        let t0 = Instant::now();
        while t0.elapsed() < READY_TIMEOUT {
            if server::is_ready() {
                let external = st.lock().unwrap().external;
                set_title(&app, if external { " (external server)" } else { "" });
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.navigate(tauri::Url::parse(server::URL).unwrap());
                }
                return;
            }
            // child died early → stop waiting
            if let Some(c) = st.lock().unwrap().child.as_mut() {
                if matches!(c.try_wait(), Ok(Some(_))) { break; }
            }
            std::thread::sleep(Duration::from_secs(1));
        }
        let msg = "ComfyUI 沒有在時限內就緒，請看 Server ▸ Show Log Window".to_string();
        server::push_log(&app, &st, format!("[app] {msg}"));
        let _ = app.emit("server-failed", &msg);
        set_title(&app, " (failed)");
    });
}

fn restart(app: &AppHandle) {
    let st: State = app.state::<State>().inner().clone();
    if st.lock().unwrap().external {
        app.dialog().message("目前接的是外部啟動的伺服器（不是這個 App 開的），不會由 App 重啟。請在終端機那邊處理。").kind(MessageDialogKind::Info).show(|_| {});
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        server::stop(&app, &st);
        show_splash(&app);
        launch(app);
    });
}

fn find_uv() -> Option<String> {
    let home = std::env::var("HOME").unwrap_or_default();
    [format!("{home}/.local/bin/uv"), "/opt/homebrew/bin/uv".into(), "/usr/local/bin/uv".into()]
        .into_iter()
        .find(|p| std::path::Path::new(p).exists())
}

fn update(app: &AppHandle) {
    let st: State = app.state::<State>().inner().clone();
    if st.lock().unwrap().external {
        app.dialog().message("目前接的是外部伺服器，請先關掉它再用 App 更新。").kind(MessageDialogKind::Info).show(|_| {});
        return;
    }
    show_log(app);
    let app = app.clone();
    std::thread::spawn(move || {
        server::stop(&app, &st);
        show_splash(&app);
        let dir = st.lock().unwrap().dir.clone();
        let mut ok = server::run_logged(&app, &st, "git", &["pull", "--ff-only"]);
        if let Ok(rd) = std::fs::read_dir(dir.join("custom_nodes")) {
            for e in rd.flatten().filter(|e| e.path().join(".git").exists()) {
                let p = e.path().display().to_string();
                ok &= server::run_logged(&app, &st, "git", &["-C", &p, "pull", "--ff-only"]);
            }
        }
        match find_uv() {
            Some(uv) => {
                let py = dir.join(".venv/bin/python").display().to_string();
                ok &= server::run_logged(&app, &st, &uv, &["pip", "install", "--python", &py, "-r", "requirements.txt"]);
                let _ = std::fs::remove_dir_all(dir.join(".uv-cache"));
            }
            None => server::push_log(&app, &st, "[app] 找不到 uv，略過 requirements 更新（pip 依賴可能過期）".into()),
        }
        server::push_log(&app, &st, if ok { "[app] 更新完成，重新啟動伺服器".into() } else { "[app] 更新過程有錯誤（見上方），仍嘗試啟動".into() });
        launch(app);
    });
}

fn show_log(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("log") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let app_menu = Submenu::with_items(app, "ComfyUI", true, &[
        &PredefinedMenuItem::about(app, None, None)?,
        &PredefinedMenuItem::separator(app)?,
        &PredefinedMenuItem::hide(app, None)?,
        &PredefinedMenuItem::quit(app, None)?,
    ])?;
    let server_menu = Submenu::with_items(app, "Server", true, &[
        &MenuItem::with_id(app, "restart", "Restart Server", true, Some("CmdOrCtrl+R"))?,
        &MenuItem::with_id(app, "log", "Show Log Window", true, Some("CmdOrCtrl+L"))?,
        &PredefinedMenuItem::separator(app)?,
        &MenuItem::with_id(app, "args", "Edit Startup Args…", true, None::<&str>)?,
        &MenuItem::with_id(app, "copyurl", "Copy URL (127.0.0.1:8188)", true, None::<&str>)?,
    ])?;
    let update_menu = Submenu::with_items(app, "Update", true, &[
        &MenuItem::with_id(app, "update", "Update ComfyUI (git pull)…", true, None::<&str>)?,
    ])?;
    let edit_menu = Submenu::with_items(app, "Edit", true, &[
        &PredefinedMenuItem::undo(app, None)?, &PredefinedMenuItem::redo(app, None)?, &PredefinedMenuItem::separator(app)?,
        &PredefinedMenuItem::cut(app, None)?, &PredefinedMenuItem::copy(app, None)?, &PredefinedMenuItem::paste(app, None)?, &PredefinedMenuItem::select_all(app, None)?,
    ])?;
    let window_menu = Submenu::with_items(app, "Window", true, &[
        &PredefinedMenuItem::minimize(app, None)?, &PredefinedMenuItem::fullscreen(app, None)?,
    ])?;
    Menu::with_items(app, &[&app_menu, &edit_menu, &server_menu, &update_menu, &window_menu])
}

pub fn run() {
    let state: State = Arc::new(Mutex::new(Server { dir: server::comfy_dir(), child: None, external: false, log: Default::default() }));
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![get_log, get_status])
        .setup(|app| {
            let handle = app.handle().clone();
            handle.set_menu(build_menu(&handle)?)?;
            handle.on_menu_event(|app, ev| match ev.id().as_ref() {
                "restart" => restart(app),
                "log" => show_log(app),
                "args" => server::open_args_file(&app.state::<State>().lock().unwrap().dir),
                "copyurl" => { let _ = app.clipboard().write_text(server::URL); }
                "update" => update(app),
                _ => {}
            });
            WebviewWindowBuilder::new(app, "log", WebviewUrl::App("log.html".into()))
                .title("ComfyUI Server Log")
                .inner_size(900.0, 600.0)
                .visible(false)
                .build()?;
            set_title(&handle, " (starting…)");
            launch(handle);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                match window.label() {
                    "log" => { api.prevent_close(); let _ = window.hide(); }
                    _ => { // main window: stop server, then quit the whole app
                        api.prevent_close();
                        let app = window.app_handle().clone();
                        let st: State = app.state::<State>().inner().clone();
                        server::stop(&app, &st);
                        app.exit(0);
                    }
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building ComfyUI.app")
        .run(|app, event| {
            // Cmd+Q / menu quit → ExitRequested; AppleEvent quit (osascript, Dock) → only Exit. stop() is idempotent.
            if matches!(event, RunEvent::ExitRequested { .. } | RunEvent::Exit) {
                let st: State = app.state::<State>().inner().clone();
                server::stop(app, &st);
            }
        });
}
