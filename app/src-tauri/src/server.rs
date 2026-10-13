//! ComfyUI server process: spawn / stop / log ring buffer / readiness.
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Emitter};

pub const PORT: u16 = 8188;
pub const URL: &str = "http://127.0.0.1:8188";
const RING: usize = 2000;

pub struct Server {
    pub dir: PathBuf,
    pub child: Option<Child>,
    /// true when 8188 was already served by something we did not start
    pub external: bool,
    pub log: VecDeque<String>,
}

pub type State = Arc<Mutex<Server>>;

pub fn comfy_dir() -> PathBuf {
    std::env::var_os("COMFYUI_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join("GitHub/ComfyUI"))
}

pub fn dir_ok(dir: &PathBuf) -> bool {
    dir.join(".venv/bin/python").exists() && dir.join("main.py").exists()
}

pub fn port_open() -> bool {
    let addr: SocketAddr = ([127, 0, 0, 1], PORT).into();
    TcpStream::connect_timeout(&addr, Duration::from_millis(300)).is_ok()
}

/// GET /system_stats and require a 200 — "port open" alone is not "ready".
pub fn is_ready() -> bool {
    let addr: SocketAddr = ([127, 0, 0, 1], PORT).into();
    let Ok(mut s) = TcpStream::connect_timeout(&addr, Duration::from_millis(500)) else { return false };
    let _ = s.set_read_timeout(Some(Duration::from_secs(3)));
    if s.write_all(b"GET /system_stats HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n").is_err() { return false }
    let mut buf = [0u8; 64];
    match s.read(&mut buf) {
        Ok(n) => String::from_utf8_lossy(&buf[..n]).starts_with("HTTP/1.1 200") || String::from_utf8_lossy(&buf[..n]).starts_with("HTTP/1.0 200"),
        Err(_) => false,
    }
}

fn extra_args(dir: &PathBuf) -> Vec<String> {
    std::fs::read_to_string(dir.join("user/app_args.txt"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .flat_map(|l| l.split_whitespace().map(String::from).collect::<Vec<_>>())
        .collect()
}

pub fn push_log(app: &AppHandle, st: &State, line: String) {
    let dir = {
        let mut s = st.lock().unwrap();
        if s.log.len() >= RING { s.log.pop_front(); }
        s.log.push_back(line.clone());
        s.dir.clone()
    };
    if line.starts_with("[app]") {
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(dir.join("user/app.log")) { let _ = writeln!(f, "{line}"); }
    }
    let _ = app.emit("server-log", &line);
}

/// Spawn the server (no-op if external). Returns Err with a message for the splash page.
pub fn start(app: &AppHandle, st: &State) -> Result<(), String> {
    let dir = st.lock().unwrap().dir.clone();
    if !dir_ok(&dir) {
        return Err(format!("找不到 ComfyUI 安裝：{}\n需要 .venv/bin/python 與 main.py（設定 COMFYUI_DIR 可改路徑）", dir.display()));
    }
    if port_open() {
        st.lock().unwrap().external = true;
        push_log(app, st, format!("[app] 127.0.0.1:{PORT} 已有伺服器在跑，直接接上（離開時不會關閉它）"));
        return Ok(());
    }
    let _ = std::fs::create_dir_all(dir.join("user"));
    let log_path = dir.join("user/app.log");
    let mut args = vec!["main.py".into(), "--listen".into(), "127.0.0.1".into(), "--port".into(), PORT.to_string()];
    args.extend(extra_args(&dir));
    push_log(app, st, format!("[app] {} {}", dir.join(".venv/bin/python").display(), args.join(" ")));

    // Run python under a tiny sh watchdog: if the app process (our pid) disappears for any reason
    // (crash, force-quit), the watchdog kills the whole group so ComfyUI is never orphaned.
    let script = "python=\"$1\"; app=\"$2\"; shift 2; \"$python\" \"$@\" & p=$!; \
                  while kill -0 \"$app\" 2>/dev/null && kill -0 \"$p\" 2>/dev/null; do sleep 1; done; \
                  kill -TERM -- -$$ 2>/dev/null; wait $p";
    let mut child = Command::new("/bin/sh")
        .arg("-c").arg(script).arg("comfyui-watchdog")
        .arg(dir.join(".venv/bin/python")).arg(std::process::id().to_string())
        .args(&args)
        .current_dir(&dir)
        .env("PYTHONUNBUFFERED", "1")
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("無法啟動 python：{e}"))?;

    let mut file = OpenOptions::new().create(true).append(true).open(&log_path).map_err(|e| format!("無法寫入 {}: {e}", log_path.display()))?;
    let _ = writeln!(file, "\n# ComfyUI.app launch {:?}", std::time::SystemTime::now());
    let file = Arc::new(Mutex::new(file));
    for pipe in [child.stdout.take().map(|p| Box::new(p) as Box<dyn Read + Send>), child.stderr.take().map(|p| Box::new(p) as Box<dyn Read + Send>)].into_iter().flatten() {
        let (app, st, file) = (app.clone(), st.clone(), file.clone());
        std::thread::spawn(move || {
            for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                if let Ok(mut f) = file.lock() { let _ = writeln!(f, "{line}"); }
                push_log(&app, &st, line);
            }
        });
    }
    let mut s = st.lock().unwrap();
    s.external = false;
    s.child = Some(child);
    Ok(())
}

/// SIGTERM the whole process group, wait ≤5 s, then SIGKILL. No-op if external / not running.
pub fn stop(app: &AppHandle, st: &State) {
    let child = { st.lock().unwrap().child.take() };
    let Some(mut child) = child else { return };
    let pid = child.id() as i32;
    push_log(app, st, format!("[app] stopping server (pgid {pid})"));
    unsafe { libc::kill(-pid, libc::SIGTERM); }
    for _ in 0..50 {
        if matches!(child.try_wait(), Ok(Some(_))) { return; }
        std::thread::sleep(Duration::from_millis(100));
    }
    unsafe { libc::kill(-pid, libc::SIGKILL); }
    let _ = child.wait();
    push_log(app, st, "[app] server killed".into());
}

/// Run a shell command inside the ComfyUI dir, streaming output to the log. Returns success.
pub fn run_logged(app: &AppHandle, st: &State, program: &str, args: &[&str]) -> bool {
    let dir = st.lock().unwrap().dir.clone();
    push_log(app, st, format!("[app] $ {program} {}", args.join(" ")));
    let Ok(mut child) = Command::new(program).args(args).current_dir(&dir).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn() else {
        push_log(app, st, format!("[app] 無法執行 {program}"));
        return false;
    };
    let mut handles = vec![];
    for pipe in [child.stdout.take().map(|p| Box::new(p) as Box<dyn Read + Send>), child.stderr.take().map(|p| Box::new(p) as Box<dyn Read + Send>)].into_iter().flatten() {
        let (app, st) = (app.clone(), st.clone());
        handles.push(std::thread::spawn(move || {
            for line in BufReader::new(pipe).lines().map_while(Result::ok) { push_log(&app, &st, line); }
        }));
    }
    for h in handles { let _ = h.join(); }
    child.wait().map(|s| s.success()).unwrap_or(false)
}

pub fn open_args_file(dir: &PathBuf) {
    let p = dir.join("user/app_args.txt");
    if !p.exists() {
        let _ = std::fs::create_dir_all(dir.join("user"));
        let _ = OpenOptions::new().create(true).write(true).open(&p).and_then(|mut f| {
            f.write_all("# ComfyUI.app 額外啟動參數，一行一個。改完用選單 Server ▸ Restart Server。\n--use-pytorch-cross-attention\n# --fp16-vae\n# --preview-method auto\n".as_bytes())
        });
    }
    let _ = Command::new("open").arg("-t").arg(&p).spawn();
}
