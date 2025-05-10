// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

prpr::tl_file!("main" mtl);

mod common;
mod ipc;
mod preview;
mod render;
mod task;

use anyhow::{bail, Context, Result};
use common::{
    collect_chart_files, create_zip, ensure_dir, get_presets_json_file, get_presets_toml_file, get_rpe_dir, output_dir, respack_dir, save_presets, Config, Extra, CONFIG_DIR, DATA_DIR
};
use fs4::tokio::AsyncFileExt;
use macroquad::prelude::set_pc_assets_folder;
use prpr::{
    fs::{self, FileSystem},
    info::ChartInfo,
};
use render::{find_ffmpeg, RenderConfig, RenderParams, ENCODER_LIST_AVC, ENCODER_LIST_HEVC};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::File,
    future::Future,
    io::{BufRead, BufReader},
    ops::DerefMut,
    path::{Path, PathBuf},
    process::Stdio,
    sync::OnceLock,
    time::SystemTime,
};
use task::{TaskQueue, TaskView};
use tauri::{ipc::InvokeError, Manager, State, WindowEvent};
use tokio::{io::AsyncWriteExt, process::Command};

static ASSET_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOCK_FILE: OnceLock<tokio::fs::File> = OnceLock::new();

#[inline]
async fn wrap_async<R>(f: impl Future<Output = Result<R>>) -> Result<R, InvokeError> {
    f.await.map_err(|e| {
        eprintln!("{e:?}");
        InvokeError::from_anyhow(e)
    })
}

async fn run_wrapped(f: impl Future<Output = Result<()>>) {
    if let Err(err) = f.await {
        eprintln!("{err:?}");
        exit_program(1);
    }
    exit_program(0);
}

fn hide_cmd() {
    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    {
        //unsafe { winapi::um::wincon::FreeConsole() };
        unsafe {
            use std::ptr::null_mut;
            use winapi::um::wincon::GetConsoleWindow;
            use winapi::um::winuser::{ShowWindow, SW_HIDE, SW_MINIMIZE};
            let console_window = GetConsoleWindow();
            if console_window != null_mut() {
                ShowWindow(console_window, SW_MINIMIZE);
            }
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> Result<()> {
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

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(TaskQueue::new())
        .invoke_handler(tauri::generate_handler![
            is_the_only_instance,
            exit_program,
            open_output_folder,
            open_in_folder,
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
            read_config,
            save_config,
            test_output_dir,
            set_rpe_dir,
            unset_rpe_dir,
            get_rpe_charts,
            open_app_folder,
            test_ffmpeg,
            test_ffmpeg_filter,
            get_encoder,
            test_encoder,
            export_pez,
            delete_path,
            delete_autosave,
            save_info,
            read_info,
        ])
        .on_window_event(|_, event| match event {
            //WindowEvent::CloseRequested { api, .. } => {
            WindowEvent::CloseRequested { .. } => {
                /*event
                .window()
                .app_handle()
                .tray_handle()
                .get_item("toggle")
                .set_title(mtl!("tray-show"))
                .unwrap();*/
                exit_program(0);
                //event.window().hide().unwrap();
                //api.prevent_close();
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    let resolver = app.path();
    let exe = std::env::current_exe()?;
    let exe_dir = exe.parent().unwrap();

    let cache_dir = ensure_dir(
        resolver
            .app_cache_dir()
            .unwrap_or_else(|_| exe_dir.to_owned()),
    );

    CONFIG_DIR
        .set(ensure_dir(
            resolver
                .app_config_dir()
                .unwrap_or_else(|_| exe_dir.to_owned()),
        ))
        .unwrap();
    DATA_DIR
        .set(ensure_dir(
            resolver
                .app_data_dir()
                .unwrap_or_else(|_| exe_dir.to_owned()),
        ))
        .unwrap();

    // let asset_dir = resolver.resolve("assets", BaseDirectory::Config).unwrap();
    let asset_dir = exe_dir.join("assets");
    ASSET_PATH.set(asset_dir.clone()).unwrap();
    set_pc_assets_folder(&asset_dir.display().to_string());

    if std::env::args().len() > 1 {
        match std::env::args().nth(1).as_deref().unwrap_or_default() {
            "help" | "--help" | "-help" | "/help" | "-h" | "?" | "--?" | "-?" | "/?" => {
                println!("Usage: phi-recorder --render <input file> [options]");
                println!("Options:");
                println!("  --output <file/path>    Output file");
                println!("  --config <file/json>    Config");
                exit_program(0);
            }
            "render" => {
                run_wrapped(render::main(false)).await;
            }
            "preview" | "play" => {
                run_wrapped(preview::main(false, false)).await;
            }
            "tweakoffset" => {
                run_wrapped(preview::main(false, true)).await;
            }
            "--render" => {
                run_wrapped(render::main(true)).await;
            }
            "--preview" | "--play" => {
                run_wrapped(preview::main(true, false)).await;
            }
            "--tweakoffset" => {
                run_wrapped(preview::main(true, true)).await;
            }
            cmd => {
                eprintln!("Command: {cmd:?}");
                let args = std::env::args().nth(1).unwrap_or_default();
                let path = Path::new(&args);
                if path.is_file() && (args.contains(".pez") || args.contains(".zip"))
                    || path.is_dir()
                {
                    println!("Find a valid path, start preview");
                    let mut child = Command::new(std::env::current_exe()?)
                        .arg("--preview")
                        .arg(args)
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn()?;
                    let status = child.wait().await?;
                    exit_program(status.code().unwrap_or_default());
                } else {
                    exit_program(1);
                }
            }
        }
    } else {
        hide_cmd();
    }

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

    app.run(|_, _| {});

    Ok(())
}

#[tauri::command]
fn is_the_only_instance() -> bool {
    LOCK_FILE.get().is_some()
}

#[tauri::command]
fn exit_program(code: i32) {
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
    std::process::exit(code);
}

#[tauri::command]
fn open_output_folder() -> Result<(), InvokeError> {
    (|| {
        let path = output_dir().unwrap();
        println!("Opening output folder: {}", path.display());
        open::that_detached(path)?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn open_in_folder(path: &Path) -> Result<(), InvokeError> {
    (move || {
        println!("Open in folder: {}", path.display());
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
            Command::new("explorer").arg(path).spawn()?;
        }

        #[cfg(target_os = "macos")]
        {
            Command::new("open").arg(path).spawn()?;
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
                    &format!("'file://{}'", path.to_str().unwrap()),
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
async fn preview_tweakoffset(params: RenderParams) -> Result<Option<f32>, InvokeError> {
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
        let mut offset = None;

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
                        offset = Some(new_offset);
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
        stdin.write_all(info.as_bytes()).await?;
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

#[tauri::command]
async fn get_presets() -> Result<HashMap<String, RenderConfig>, InvokeError> {
    (|| {
        let toml_file = get_presets_toml_file()?;
        if toml_file.exists() {
            let presets: HashMap<String, RenderConfig> =
                toml::from_str(&std::fs::read_to_string(toml_file)?)?;
            return Ok(presets);
        }

        // Compatible with old versions
        let json_file = get_presets_json_file()?;
        if json_file.exists() {
            let presets: HashMap<String, RenderConfig> =
                serde_json::from_reader(BufReader::new(File::open(json_file)?))?;
            return Ok(presets);
        }

        Ok(HashMap::new())
    })()
    .map_err(InvokeError::from_anyhow)
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

#[tauri::command]
async fn read_config() -> Result<Config, InvokeError> {
    common::read_config().map_err(InvokeError::from_anyhow)
}

#[tauri::command]
async fn save_config(config: Config) -> Result<(), InvokeError> {
    common::save_config(config).map_err(InvokeError::from_anyhow)
}

#[tauri::command]
async fn test_output_dir(dir: PathBuf) -> Result<(), InvokeError> {
    common::test_output_dir(dir).map_err(InvokeError::from_anyhow)
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

#[tauri::command]
fn set_rpe_dir(path: PathBuf, save: bool) -> Result<(), InvokeError> {
    (|| {
        if !path.is_dir()
            || ["PhiEdit.exe", "Resources"]
                .iter()
                .any(|it| !path.join(*it).exists())
        {
            bail!(mtl!("not-valid-rpe"));
        }
        if save {
            common::set_rpe_dir(Some(path))?;
        }
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn unset_rpe_dir() -> Result<(), InvokeError> {
    (|| {
        common::set_rpe_dir(None)?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn get_rpe_charts() -> Result<Option<Vec<RPEChartInfo>>, InvokeError> {
    (|| {
        let Ok(dir) = get_rpe_dir() else {
            return Ok(None);
        };
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
                    let metadata = path.join(chart.take()?).metadata().ok();

                    let modified = metadata
                        .and_then(|m| m.modified().ok())
                        .unwrap_or(SystemTime::UNIX_EPOCH);

                    results.push(RPEChartInfo {
                        name: name.take()?,
                        id,
                        path: path.display().to_string(),
                        illustration: path.join(illustration.take()?).display().to_string(),
                        charter: charter.take()?,
                        modified,
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
                let Some((key, value)) = line.split_once(':') else {
                    continue;
                };
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
            use regex::Regex;
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
                println!("Found chart folder: {}", folder.file_name().unwrap_or_default().to_string_lossy());
                if !folder.join("info.txt").exists() {
                    println!("Not found info.txt");
                    let folder_name = folder.file_name().unwrap_or_default().to_string_lossy().to_string();
                    results.push(RPEChartInfo {
                        name: folder_name,
                        id: "Empty folder".to_string(),
                        path: folder.display().to_string(),
                        illustration: "".to_string(),
                        charter: "".to_string(),
                        modified: SystemTime::UNIX_EPOCH,
                    });
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
                    let Some((key, value)) = line.split_once(':') else {
                        continue;
                    };
                    *(match key {
                        "Name" => &mut name,
                        "Path" => &mut id,
                        "Chart" => &mut chart,
                        "Picture" => &mut illustration,
                        "Charter" => &mut charter,
                        _ => continue,
                    }) = Some(value.trim().to_owned());
                    print!("{}, ", value.trim());
                }
                println!();
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

#[tauri::command]
fn test_ffmpeg() -> Result<bool, InvokeError> {
    (|| Ok(find_ffmpeg()?.is_some()))().map_err(InvokeError::from_anyhow)
}

#[tauri::command]
async fn test_ffmpeg_filter() -> bool {
    let Ok(Some(ffmpeg)) = find_ffmpeg() else {
        return false;
    };
    eprintln!("ffmpeg: {}", &ffmpeg);

    let output = Command::new(&ffmpeg)
        .arg("-filters")
        .output()
        .await
        .expect("failed test filter");

    let banner = String::from_utf8(output.stderr).unwrap_or_default();
    if !banner.contains("--enable-libsoxr") {
        eprintln!("Missing lib: libsoxr, Place update FFmpeg to full version");
        return false;
    }

    let filter = String::from_utf8(output.stdout).unwrap_or_default();
    let filter_required = ["aresample", "alimiter", "acompressor", "volume"];
    for i in filter_required {
        if !filter.contains(i) {
            eprintln!("Missing lib: {}, Place update FFmpeg to full version", i);
            return false;
        }
    }
    return true;
}

#[tauri::command]
async fn get_encoder(hevc: bool) -> Result<Option<String>, InvokeError> {
    (|| {
        let Some(ffmpeg) = find_ffmpeg()? else {
            bail!("FFmpeg not found")
        };
        let config = RenderConfig {
            hevc,
            ..RenderConfig::default()
        };
        let encoder_list = if config.hevc {
            ENCODER_LIST_HEVC
        } else {
            ENCODER_LIST_AVC
        };
        Ok(render::get_encoder(&ffmpeg, &config, encoder_list, false))
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
async fn test_encoder(encoder: String) -> Result<bool, InvokeError> {
    (|| {
        let Some(ffmpeg) = find_ffmpeg()? else {
            bail!("FFmpeg not found")
        };
        Ok(render::test_encoder(ffmpeg, encoder))
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
async fn export_pez(chart_path: String, output_path: String) -> Result<(), InvokeError> {
    wrap_async(async move {
        println!("Exporting PEZ: {} -> {}", chart_path, output_path);
        let chart_path = PathBuf::from(chart_path);
        let output_path = PathBuf::from(output_path);

        if !chart_path.exists() || !chart_path.is_dir() {
            bail!("Not a directory");
        }

        let mut files = collect_chart_files(chart_path.clone(), chart_path.clone())?;
        let res_path = chart_path.parent().unwrap().join("shaders").join("pr");

        let extra_file = chart_path.join("extra.json");
        if extra_file.exists() {
            let mut shaders = Vec::new();
            let extra: Extra = serde_json::from_str(&std::fs::read_to_string(extra_file)?)?;
            for effect in extra.effects {
                if let Some(shader) = effect.shader.strip_prefix("/") {
                    let shader = shader.to_string();
                    if shader.ends_with("_pr.glsl") && !shaders.contains(&shader) {
                        shaders.push(shader);
                    }
                }
            }
            for shader in shaders {
                let shader_path = chart_path.join(&shader);
                let rpe_shader_path = res_path.join(&shader);
                if !shader_path.exists() && rpe_shader_path.exists() {
                    files.insert(shader, rpe_shader_path);
                }
            }
        }
        println!("files: {:?}", files);

        create_zip(output_path, files).await?;
        Ok(())
    }).await
}

#[tauri::command]
async fn delete_path(path: String) -> Result<(), InvokeError> {
    wrap_async(async move {
        let path = PathBuf::from(path);
        if path.exists() && path.is_dir() {
            tokio::fs::remove_dir_all(path).await?;
        } else {
            bail!("Not a directory");
        }
        Ok(())
    }).await
}

#[tauri::command]
async fn delete_autosave(path: String) -> Result<(), InvokeError> {
    wrap_async(async move {
        let path = PathBuf::from(path);
        if path.exists() && path.is_dir() {
            let mut entries = tokio::fs::read_dir(&path).await?;

            while let Some(entry) = entries.next_entry().await? {
                let file_path = entry.path();
                if let Some(file_name) = file_path.file_name().and_then(|s| s.to_str()) {
                    if file_name.starts_with("AutoSave_") {
                        tokio::fs::remove_file(file_path).await?;
                    }
                }
            }
        } else {
            bail!("Not a directory");
        }
        Ok(())
    }).await
}

#[tauri::command]
async fn save_info(path: String, info: ChartInfo) -> Result<(), InvokeError> {
    wrap_async(async move {
        let file = PathBuf::from(path);
        let string = serde_yaml::to_string(&info)?;
        std::fs::write(file, string)?;
        Ok(())
    }).await
}

#[tauri::command]
async fn read_info(path: String) -> Result<ChartInfo, InvokeError> {
    wrap_async(async move {
        let file = PathBuf::from(path);
        let info = serde_yaml::from_reader(BufReader::new(std::fs::File::open(file)?))?;
        Ok(info)
    }).await
}