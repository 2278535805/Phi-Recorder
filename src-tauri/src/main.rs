// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

prpr::tl_file!("main" mtl);

mod common;
mod ipc;
mod preview;
mod render;
mod task;
mod icon;

use anyhow::{bail, Context, Result};
use common::{ensure_dir, respack_dir, output_dir, CONFIG_DIR, DATA_DIR};
use fs4::tokio::AsyncFileExt;
use macroquad::{miniquad::conf::Icon, prelude::set_pc_assets_folder};
use prpr::{
    fs::{self, FileSystem},
    info::ChartInfo,
};
use render::{find_ffmpeg, RenderConfig, RenderParams};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::File,
    future::Future,
    io::{BufRead, BufReader, BufWriter},
    ops::DerefMut,
    path::{Path, PathBuf},
    process::Stdio,
    sync::OnceLock,
    time::SystemTime,
};
use task::{TaskQueue, TaskView};
use tauri::{
    CustomMenuItem, InvokeError, Manager, State, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowEvent
};
use tokio::{io::AsyncWriteExt, process::Command};
use icon::{
    BIG_ICON, ICON, SMALL_ICON
};

static ASSET_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOCK_FILE: OnceLock<tokio::fs::File> = OnceLock::new();

#[inline]
async fn wrap_async<R>(f: impl Future<Output = Result<R>>) -> Result<R, InvokeError> {
    f.await.map_err(|e| {
        eprintln!("{e:?}");
        InvokeError::from_anyhow(e)
    })
}

pub fn build_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Phi Recorder".to_string(),
        window_width: 1280,
        window_height: 720,
        icon: Some(Icon {
            medium: ICON,
            big: BIG_ICON,
            small: SMALL_ICON
        }),
        headless: !matches!(
            std::env::args().skip(1).next().as_deref(),
            Some("tweakoffset") | Some("preview") | Some("play") | Some("--tweakoffset") | Some("--preview") | Some("--play")
        ),
        ..Default::default()
    }
}

async fn run_wrapped(f: impl Future<Output = Result<()>>) -> ! {
    if let Err(err) = f.await {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
    std::process::exit(0);
}

fn hide_cmd() {
    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    {
        //unsafe { winapi::um::wincon::FreeConsole() };
        unsafe {
            use std::ptr::null_mut;
            use winapi::um::winuser::{ShowWindow, SW_HIDE, SW_MINIMIZE};
            use winapi::um::wincon::GetConsoleWindow;
            let console_window = GetConsoleWindow();
            if console_window != null_mut() {
                ShowWindow(console_window, SW_MINIMIZE);
            }
        }
    }
}

#[macroquad::main(build_conf)]
async fn main() -> Result<()> {
    /*use chrono::prelude::*;
    let now = Utc::now();
    let target_date = Utc.with_ymd_and_hms(2025, 2, 5, 0, 0, 0).unwrap();
    if now >= target_date {
        panic!("Outdated version!");
    }*/

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();

    if std::env::args().len() > 1 {
        match std::env::args().nth(1).as_deref() {
            Some("render") => {
                run_wrapped(render::main(false)).await;
            }
            Some("preview") | Some("play") => {
                run_wrapped(preview::main(false, false)).await;
            }
            Some("tweakoffset") => {
                run_wrapped(preview::main(false, true)).await;
            }
            Some("--render") => {
                run_wrapped(render::main(true)).await;
            }
            Some("--preview") | Some("--play") => {
                run_wrapped(preview::main(true, false)).await;
            }
            Some("--tweakoffset") => {
                run_wrapped(preview::main(true, true)).await;
            }
            cmd => {
                eprintln!("Command: {cmd:?}");
                let args = std::env::args().nth(1).unwrap_or_default();
                let path = Path::new(&args);
                if path.is_file() && (args.contains(".pez") || args.contains(".zip")) || path.is_dir() {
                    println!("Find a valid path, start preview");
                    let mut child = Command::new(std::env::current_exe()?)
                        .arg("--preview")
                        .arg(args)
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn()?;
                    let status = child.wait().await?;
                    std::process::exit(status.code().unwrap_or_default());
                } else {
                    std::process::exit(1);
                }
            }
        }
    } else {
        hide_cmd();
    }

    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("toggle".to_owned(), mtl!("tray-hide")))
        .add_item(CustomMenuItem::new("tasks".to_owned(), mtl!("tray-tasks")))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_owned(), mtl!("tray-quit")));
    let app = tauri::Builder::default()
        .system_tray(SystemTray::new().with_menu(tray_menu))
        .manage(TaskQueue::new())
        .invoke_handler(tauri::generate_handler![
            is_the_only_instance,
            exit_program,
            show_folder,
            show_in_folder,
            open_file,
            preview_chart,
            preview_tweakoffset,
            preview_play,
            parse_chart,
            post_render,
            get_tasks,
            cancel_task,
            get_respacks,
            open_respack_folder,
            get_presets,
            add_preset,
            remove_preset,
            set_rpe_dir,
            unset_rpe_dir,
            get_rpe_charts,
            test_ffmpeg,
            open_app_folder,
        ])
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                let window = app.get_window("main").unwrap();
                let visible = window.is_visible().unwrap();
                let minimizable = window.is_minimizable().unwrap();
                match id.as_str() {
                    "toggle" => {
                        app.tray_handle()
                            .get_item(&id)
                            .set_title(if visible {
                                mtl!("tray-show")
                            } else {
                                mtl!("tray-hide")
                            })
                            .unwrap();
                        if visible {
                            window.hide().unwrap();
                        } else {
                            window.show().unwrap();
                            window.unminimize().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                    "tasks" => {
                        if !visible || minimizable {
                            window.show().unwrap();
                            window.unminimize().unwrap();
                        }
                        window.eval("window.goto('tasks')").unwrap();
                        window.set_focus().unwrap();
                    }
                    "quit" => {
                        exit_program();
                    }
                    _ => {}
                }
            }
            SystemTrayEvent::LeftClick { .. } => {
                let window = app.get_window("main").unwrap();
                let visible = window.is_visible().unwrap();
                app.tray_handle()
                            .get_item("toggle")
                            .set_title(if visible {
                                window.hide().unwrap();
                                mtl!("tray-show")
                            } else {
                                window.show().unwrap();
                                window.unminimize().unwrap();
                                window.set_focus().unwrap();
                                mtl!("tray-hide")
                            })
                            .unwrap();
            }
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            //WindowEvent::CloseRequested { api, .. } => {
            WindowEvent::CloseRequested { .. } => {
                /*event
                    .window()
                    .app_handle()
                    .tray_handle()
                    .get_item("toggle")
                    .set_title(mtl!("tray-show"))
                    .unwrap();*/
                exit_program();
                //event.window().hide().unwrap();
                //api.prevent_close();
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    let resolver = app.path_resolver();
    let exe = std::env::current_exe()?;
    let exe_dir = exe.parent().unwrap();

    let cache_dir = ensure_dir(
        resolver
            .app_cache_dir()
            .unwrap_or_else(|| exe_dir.to_owned()),
    );
    let lock_file = tokio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(cache_dir.join("app.lock"))
        .await?;
    if lock_file.try_lock_exclusive().is_ok() {
        LOCK_FILE.set(lock_file).unwrap();
    } else {
        eprintln!("Lock failed");
    }

    CONFIG_DIR
        .set(ensure_dir(
            resolver
                .app_config_dir()
                .unwrap_or_else(|| exe_dir.to_owned()),
        ))
        .unwrap();
    DATA_DIR
        .set(ensure_dir(
            resolver
                .app_data_dir()
                .unwrap_or_else(|| exe_dir.to_owned()),
        ))
        .unwrap();

    let asset_dir = resolver.resolve_resource("assets").unwrap();
    ASSET_PATH.set(asset_dir.clone()).unwrap();
    set_pc_assets_folder(&asset_dir.display().to_string());

    app.run(|_, _| {});

    Ok(())
}

#[tauri::command]
fn is_the_only_instance() -> bool {
    LOCK_FILE.get().is_some()
}

#[tauri::command]
fn exit_program() {
    /*#[cfg(target_os = "windows")]
    {
        use sysinfo::{ProcessExt, System, SystemExt, PidExt};
        let current_exe = std::env::current_exe().unwrap();
        let exe_name = current_exe.file_name().unwrap().to_str().unwrap();
        let mut system = System::new_all();
        system.refresh_processes();
        for (pid, process) in system.processes() {
            if process.name() == exe_name {
                if pid.as_u32() == std::process::id() {
                    continue;
                }
                process.kill();
            }
        }
    }*/
    std::process::exit(0);
}

#[tauri::command]
fn show_folder() -> Result<(), InvokeError> {
    (|| {
        let path = output_dir().unwrap();
        println!("Opening output folder: {}", path.display());
        open::that_detached(path)?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn show_in_folder(path: &Path) -> Result<(), InvokeError> {
    (move || {
        println!("Show in folder: {}", path.display());
        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .args(["/select,", &path.display().to_string()]) // The comma after select is not a typo
                .spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("gdbus")
                .args([
                    "call",
                    "--session",
                    "--dest",
                    "org.freedesktop.FileManager1",
                    "--object-path",
                    "/org/freedesktop/FileManager1",
                    "--method",
                    "org.freedesktop.FileManager1.ShowItems",
                    &format!("['file://{}']", path.canonicalize()?.display()),
                    "",
                ])
                .spawn()?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .args(["-R", &path.display().to_string()])
                .spawn()?;
        }

        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn open_file(path: &Path) -> Result<(), InvokeError> {
    let result = (move || {
        println!("Opening file: {}", path.display());

        #[cfg(target_os = "windows")]
        {
            Command::new("explorer")
                .arg(path)
                .spawn()?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open")
                .arg(path)
                .spawn()?;
        }

        #[cfg(target_os = "linux")]
        {
            Command::new("gdbus")
                .args(&[
                    "call",
                    "--session",
                    "--dest=org.freedesktop.portal.Desktop",
                    "--object-path=/org/freedesktop/portal/desktop",
                    "--method=org.freedesktop.portal.OpenURI.OpenFile",
                    &format!("'file://{}'", path.to_str().unwrap())
                ])
                .spawn()?;
        }

        Ok(())
    })();
    
    result.map_err(InvokeError::from_anyhow)
}

#[tauri::command]
async fn parse_chart(path: &Path) -> Result<ChartInfo, InvokeError> {
    wrap_async(async move {
        let mut fs: Box<dyn FileSystem + Send + Sync + 'static> =
            fs::fs_from_file(path).with_context(|| mtl!("read-chart-failed"))?;
        let info = fs::load_info(fs.deref_mut())
            .await
            .with_context(|| mtl!("load-info-failed"))?;
        //let info1 = format!("{}\n", serde_json::to_string(&info)?);
        //println!("{}", info1);
        Ok(info)
    })
    .await
}

pub fn cmd_hidden(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let cmd = tokio::process::Command::new(program);
    /*#[cfg(target_os = "windows")] // Without terminal, there is no log
    {
        let mut cmd = cmd;
        #[cfg(not(debug_assertions))]
        cmd.creation_flags(0x08000000);
        cmd
    }
    #[cfg(not(target_os = "windows"))]*/
    cmd
}

#[tauri::command]
async fn preview_chart(params: RenderParams) -> Result<(), InvokeError> {
    wrap_async(async move {
        let mut child = cmd_hidden(std::env::current_exe()?)
            .arg("preview")
            .arg(ASSET_PATH.get().unwrap())
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        stdin
            .write_all(format!("{}\n", serde_json::to_string(&params)?).as_bytes())
            .await?;

        Ok(())
    })
    .await
}

#[tauri::command]
async fn preview_tweakoffset(params: RenderParams) -> Result<f32, InvokeError> {
    wrap_async(async move {
        let mut child = cmd_hidden(std::env::current_exe()?)
            .arg("tweakoffset")
            .arg(ASSET_PATH.get().unwrap())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        let info = format!("{}\n", serde_json::to_string(&params)?);
        stdin.write_all(info.as_bytes()).await?;

        // Read and process stdout to get the offset value
        let stdout = child.stdout.take().unwrap();
        let mut reader = tokio::io::BufReader::new(stdout);
        let mut line = String::new();
        let mut offset = 0.0f32;

        while let Ok(bytes) = tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut line).await {
            if bytes == 0 {
                break;
            }

            if line.contains("{update offset:") {
                // Extract the offset value using regex
                if let Some(offset_str) = line
                    .trim()
                    .strip_prefix("{update offset:")
                    .and_then(|s| s.strip_suffix("}"))
                {
                    if let Ok(new_offset) = offset_str.trim().parse::<f32>() {
                        println!("update offset:{}", new_offset);
                        offset = new_offset;
                        break;
                    }
                }
            }

            line.clear();
        }

        let status = child.wait().await?;
        if !status.success() {
            println!("Child process exited with {}", status);
        }

        Ok(offset)
    })
    .await
}

#[tauri::command]
async fn preview_play(params: RenderParams) -> Result<(), InvokeError> {
    wrap_async(async move {
        let mut child = cmd_hidden(std::env::current_exe()?)
            .arg("play")
            .arg(ASSET_PATH.get().unwrap())
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        let info = format!("{}\n", serde_json::to_string(&params)?);
        stdin
            .write_all(info.as_bytes())
            .await?;
        Ok(())
    })
    .await
}

#[tauri::command]
async fn post_render(queue: State<'_, TaskQueue>, params: RenderParams) -> Result<(), InvokeError> {
    wrap_async(async move {
        queue.post(params).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
async fn get_tasks(queue: State<'_, TaskQueue>) -> Result<Vec<TaskView>, InvokeError> {
    wrap_async(async move { Ok(queue.tasks().await) }).await
}

#[tauri::command]
async fn cancel_task(queue: State<'_, TaskQueue>, id: u32) -> Result<(), InvokeError> {
    queue.cancel(id).await;
    Ok(())
}

#[derive(Serialize)]
struct RespackInfo {
    name: String,
    path: String,
}
#[tauri::command]
fn get_respacks() -> Result<Vec<RespackInfo>, InvokeError> {
    (|| {
        let dir = respack_dir()?;
        let mut names: Vec<RespackInfo> = dir
            .read_dir()?
            .filter_map(|it| {
                it.ok()
                    .filter(|it| it.path().is_file())
                    .map(|it| RespackInfo {
                        name: it.file_name().to_str().unwrap().to_owned(),
                        path: it.path().canonicalize().unwrap().display().to_string(),
                    })
            })
            .collect();
        names.sort_by(|x, y| x.name.cmp(&y.name));
        Ok(names)
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn open_respack_folder() -> Result<(), InvokeError> {
    (|| {
        let path = respack_dir()?;
        println!("Opening respack folder: {}", path.display());
        open::that_detached(path)?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

fn get_presets_file() -> Result<PathBuf> {
    let file = CONFIG_DIR.get().unwrap().join("presets.json");
    if file.exists() && !file.is_file() {
        bail!("presets.json is not a file");
    }
    Ok(file)
}

#[tauri::command]
async fn get_presets() -> Result<HashMap<String, RenderConfig>, InvokeError> {
    (|| {
        let file = get_presets_file()?;
        Ok(if !file.exists() {
            HashMap::new()
        } else {
            serde_json::from_reader(BufReader::new(File::open(file)?))?
        })
    })()
    .map_err(InvokeError::from_anyhow)
}

async fn save_presets(presets: &HashMap<String, RenderConfig>) -> Result<()> {
    serde_json::to_writer(BufWriter::new(File::create(get_presets_file()?)?), presets)?;
    Ok(())
}

#[tauri::command]
async fn add_preset(name: String, config: RenderConfig) -> Result<(), InvokeError> {
    let mut presets = get_presets().await?;
    wrap_async(async move {
        if presets.insert(name, config).is_some() {
            bail!(mtl!("preset-exists"));
        }
        save_presets(&presets).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
async fn remove_preset(name: String) -> Result<(), InvokeError> {
    let mut presets = get_presets().await?;
    wrap_async(async move {
        if presets.remove(&name).is_none() {
            bail!(mtl!("preset-not-found"));
        }
        save_presets(&presets).await?;
        Ok(())
    })
    .await
}

fn rpe_dir() -> Result<Option<PathBuf>> {
    let file = CONFIG_DIR.get().unwrap().join("rpe_path.txt");
    if file.exists() {
        if !file.is_file() {
            bail!("rpe_path.txt is not a file");
        }
    } else {
        return Ok(None);
    }
    let dir: PathBuf = std::fs::read_to_string(file)?.into();
    Ok(if dir.exists() { Some(dir) } else { None })
}

#[derive(Serialize)]
pub struct RPEChartInfo {
    name: String,
    id: String,
    path: String,
    illustration: String,
    charter: String,
    #[serde(skip)]
    modified: SystemTime,
}

fn remove_verbatim_prefix(path: &PathBuf) -> PathBuf {
    let path_str = path.to_str().unwrap_or("");
    if path_str.starts_with(r"\\?\") && path_str.len() < 260 {
        PathBuf::from(&path_str[4..])
    } else {
        path.to_path_buf()
    }
}

#[tauri::command]
fn set_rpe_dir(path: PathBuf) -> Result<(), InvokeError> {
    (|| {
        if !path.is_dir()
            || ["PhiEdit.exe", "Resources"]
                .iter()
                .any(|it| !path.join(*it).exists())
        {
            bail!(mtl!("not-valid-rpe"));
        }
        let file = CONFIG_DIR.get().unwrap().join("rpe_path.txt");
        println!("Create {}", file.display());
        std::fs::write(
            file,
            remove_verbatim_prefix(&path.canonicalize()?).display().to_string().as_bytes(),
        )?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn unset_rpe_dir() -> Result<(), InvokeError> {
    (|| {
        let file = CONFIG_DIR.get().unwrap().join("rpe_path.txt");
        println!("Delete {}", file.display());
        std::fs::remove_file(file)?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn get_rpe_charts() -> Result<Option<Vec<RPEChartInfo>>, InvokeError> {
    (|| {
        let Some(dir) = rpe_dir()? else { return Ok(None) };
        let mut results = Vec::new();
        let mut name = None;
        let mut id = None;
        let mut chart = None;
        let mut illustration = None;
        let mut charter = None;
        macro_rules! commit {
            () => {
                (|| {
                    let id = id.take()?;
                    let path = dir.join("Resources").join(&id);
                    let metadata = path.join(chart.take()?).metadata();
                    results.push(RPEChartInfo {
                        name: name.take()?,
                        id,
                        path: path.display().to_string(),
                        illustration: path.join(illustration.take()?).display().to_string(),
                        charter: charter.take()?,
                        modified: metadata
                            .and_then(|it| it.modified())
                            .unwrap_or(SystemTime::UNIX_EPOCH),
                    });
                    Some(())
                })()
            };
        }

        if dir.join("Chartlist.txt").exists() {
            println!("Reading Chartlist.txt");
            for line in BufReader::new(File::open(dir.join("Chartlist.txt"))?).lines() {
                let line = line?;
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "#" {
                    commit!();
                    continue;
                }
                let Some((key, value)) = line.split_once(':') else { continue };
                *(match key {
                    "Name" => &mut name,
                    "Path" => &mut id,
                    "Chart" => &mut chart,
                    "Picture" => &mut illustration,
                    "Charter" => &mut charter,
                    _ => continue,
                }) = Some(value.trim().to_owned());
                if key == "Name" {
                    println!("Found {}", value);
                }
            }
            commit!();
        } else {
            println!("Not found Chartlist.txt, start reading folder");
            use tauri::regex::Regex;
            let onely_num = Regex::new(r"^\d+$").unwrap();
            let mut folders = Vec::new();
            let path = dir.join("Resources");
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(folder_name) = path.file_name() {
                        if onely_num.is_match(folder_name.to_str().unwrap_or("")) {
                            folders.push(path);
                        }
                    }
                }
            }
            for folder in folders {
                println!("Found chart folder: {}", folder.display());
                if !folder.join("info.txt").exists() {
                    println!("Not found info.txt, skip");
                    continue;
                }
                for line in BufReader::new(File::open(folder.join("info.txt"))?).lines() {
                    let line = line?;
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if line == "#" {
                        commit!();
                        continue;
                    }
                    let Some((key, value)) = line.split_once(':') else { continue };
                    *(match key {
                        "Name" => &mut name,
                        "Path" => &mut id,
                        "Chart" => &mut chart,
                        "Picture" => &mut illustration,
                        "Charter" => &mut charter,
                        _ => continue,
                    }) = Some(value.trim().to_owned());
                    if key == "Name" {
                        println!("Found {}", value);
                    }
                }
                commit!();
            }
        }

        results.sort_by_key(|it| it.modified);
        results.reverse();

        Ok(Some(results))
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn test_ffmpeg() -> Result<bool, InvokeError> {
    (|| Ok(find_ffmpeg()?.is_some()))().map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn open_app_folder() -> Result<(), InvokeError> {
    (|| {
        let exe_path = std::env::current_exe()?;
        let path = exe_path.parent().unwrap();
        println!("Opening current exe folder: {}", path.display());
        open::that_detached(path)?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}
