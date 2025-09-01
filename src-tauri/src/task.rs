use crate::{
    cmd_hidden,
    common::{get_output_dir, read_config},
    ipc::IPCEvent,
    render::{RenderConfig, RenderParams},
    ASSET_PATH
};
use anyhow::Result;
use chrono::Local;
use phire::{fs, info::ChartInfo};
use regex::Regex;
use serde::Serialize;
use tracing::{error, info};
use std::{
    collections::VecDeque,
    io::Write,
    path::PathBuf,
    process::Stdio,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use tempfile::NamedTempFile;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::{mpsc, Mutex, Semaphore},
    task::JoinHandle, time::sleep,
};
use humantime::format_duration;

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum TaskStatus {
    Pending,
    Loading,
    Mixing,
    MixingSfx {
        progress: f64,
    },
    Rendering {
        progress: f64,
        fps: u64,
        estimate: f64,
    },
    Done {
        duration: f64,
        output: String,
    },
    Canceled {
        output: String,
    },
    Failed {
        output: String,
    },
}

pub fn generate_filename(info: &ChartInfo, config: &RenderConfig) -> String {
    let re = Regex::new(r"%([^%]+)%").unwrap();
    let input = config.file_name_format.as_str();
    let mut output = input.to_string();

    let match_config = |key: &str| -> String {
        match key {
            "resolution"                 => format!("{}x{}", config.resolution.0, config.resolution.1),
            "ffmpeg_preset"              => config.ffmpeg_preset.clone(),
            "ending_length"              => config.ending_length.to_string(),
            "render_loading"             => config.render_loading.to_string(),
            "hires"                      => config.hires.to_string(),
            "chart_debug_line"           => config.chart_debug_line.to_string(),
            "chart_debug_note"           => config.chart_debug_note.to_string(),
            "chart_ratio"                => config.chart_ratio.to_string(),
            "all_good"                   => config.all_good.to_string(),
            "all_bad"                    => config.all_bad.to_string(),
            "fps"                        => config.fps.to_string(),
            "hardware_accel"             => config.hardware_accel.to_string(),
            "hevc"                       => config.hevc.to_string(),
            "mpeg4"                      => config.mpeg4.to_string(),
            "custom_encoder"             => config.custom_encoder.clone().unwrap_or_default(),
            "dynamic_bitrate_control"    => config.dynamic_bitrate_control.to_string(),
            "bitrate"                    => config.bitrate.clone(),
            "aggressive"                 => config.aggressive.to_string(),
            "challenge_color"            => config.challenge_color.to_string(),
            "challenge_rank"             => config.challenge_rank.to_string(),
            "fxaa"                       => config.fxaa.to_string(),
            "note_scale"                 => config.note_scale.to_string(),
            "particle"                   => config.particle.to_string(),
            "player_avatar"              => config.player_avatar.clone().unwrap_or_default(),
            "player_name"                => config.player_name.clone(),
            "player_rks"                 => config.player_rks.to_string(),
            "sample_count"               => config.sample_count.to_string(),
            "res_pack_path"              => config.res_pack_path.clone().unwrap_or_default(),
            "speed"                      => config.speed.to_string(),
            "volume_music"               => config.volume_music.to_string(),
            "volume_sfx"                 => config.volume_sfx.to_string(),
            "compression_ratio"          => config.compression_ratio.to_string(),
            "force_limit"                => config.force_limit.to_string(),
            "limit_threshold"            => config.limit_threshold.to_string(),
            "loudness_equalization"      => config.loudness_equalization.to_string(),
            "audio_mix_optimization"     => config.audio_mix_optimization.to_string(),
            "watermark"                  => config.watermark.clone(),
            "roman"                      => config.roman.to_string(),
            "chinese"                    => config.chinese.to_string(),
            "combo"                      => config.combo.clone(),
            "difficulty"                 => config.difficulty.clone(),
            "judge_offset"               => config.judge_offset.to_string(),
            "file_name_format"           => config.file_name_format.clone(),
            "render_line"                => config.render_line.to_string(),
            "render_line_extra"          => config.render_line_extra.to_string(),
            "render_note"                => config.render_note.to_string(),
            "render_double_hint"         => config.render_double_hint.to_string(),
            "render_ui_pause"            => config.render_ui_pause.to_string(),
            "render_ui_name"             => config.render_ui_name.to_string(),
            "render_ui_level"            => config.render_ui_level.to_string(),
            "render_ui_score"            => config.render_ui_score.to_string(),
            "render_ui_combo"            => config.render_ui_combo.to_string(),
            "render_ui_bar"              => config.render_ui_bar.to_string(),
            "render_bg"                  => config.render_bg.to_string(),
            "render_bg_dim"              => config.render_bg_dim.to_string(),
            "render_extra"               => config.render_extra.to_string(),
            "bg_blurriness"              => config.bg_blurriness.to_string(),
            "max_particles"              => config.max_particles.to_string(),
            "play_start_time"            => config.play_start_time.to_string(),
            "play_end_time"              => config.play_end_time.map_or_else(String::new, |v| v.to_string()),
            "fade"                       => config.fade.to_string(),
            "alpha_tint"                 => config.alpha_tint.to_string(),
            _                            => key.to_string(),
        }
    };


    let match_info = |key: &str| -> String {
        match key {
            "id"                 => info.id.map_or_else(String::new, |v| v.to_string()),
            "uploader"           => info.uploader.map_or_else(String::new, |v| v.to_string()),
            "name"               => info.name.clone(),
            "difficulty"         => info.difficulty.to_string(),
            "level"              => info.level.clone(),
            "charter"            => info.charter.clone(),
            "composer"           => info.composer.clone(),
            "illustrator"        => info.illustrator.clone(),
            "chart"              => info.chart.clone(),
            "music"              => info.music.clone(),
            "illustration"       => info.illustration.clone(),
            "preview_start"      => info.preview_start.to_string(),
            "preview_end"        => info.preview_end.map_or_else(String::new, |v| v.to_string()),
            "aspect_ratio"       => info.aspect_ratio.to_string(),
            "force_aspect_ratio" => info.force_aspect_ratio.to_string(),
            "background_dim"     => info.background_dim.to_string(),
            "line_length"        => info.line_length.to_string(),
            "offset"             => info.offset.to_string(),
            "tip"                => info.tip.clone().unwrap_or_default(),
            "tags"               => info.tags.join(","),
            "intro"              => info.intro.clone(),
            "hold_partial_cover" => info.hold_partial_cover.to_string(),
            "note_uniform_scale" => info.note_uniform_scale.to_string(),
            "score_total"        => info.score_total.to_string(),
            "created"            => info.created.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            "updated"            => info.updated.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            "chart_updated"      => info.chart_updated.map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            _                    => key.to_string(),
        }
    };


    for caps in re.captures_iter(input) {
        let whole = caps.get(0).unwrap().as_str();
        let key   = caps.get(1).unwrap().as_str();

        let replacement =
            if let Some(key) = key.strip_prefix("config.") {
                match_config(key)
            } else if let Some(key) = key.strip_prefix("info.") {
                match_info(key)
            } else {
                match key {
                    "date" => Local::now().format("%Y-%m-%d").to_string(),
                    "time" => Local::now().format("%H-%M-%S").to_string(),
                    "level_prefix" => info.level.split_whitespace().next().unwrap_or("UK").to_string(),
                    _ => whole.to_string(),
                }
            };
        output = output.replace(whole, &replacement);
    }

    fn safe_filename(name: String) -> String {
        name
            .trim()
            .chars()
            .filter(|&it| it.is_alphanumeric() || " !#$%&'()+,-.;=@[]^_`{}~".contains(it))
            .collect()
    }

    let format = if config.hires { "mov" } else { "mp4" };

    format!("{}.{}", safe_filename(output), format)
}

pub struct Task {
    id: u32,
    name: String,
    cover: NamedTempFile,
    output: PathBuf,

    params: RenderParams,
    status: Mutex<TaskStatus>,
    request_cancel: AtomicBool,
}

impl Task {
    async fn new(id: u32, params: RenderParams) -> Result<Self> {
        let mut fs = fs::fs_from_file(&params.path)?;
        let info = params.info.clone();
        let mut cover = NamedTempFile::new()?;
        cover.write_all(&fs.load_file(&info.illustration).await?)?;

        let file_name = generate_filename(&info, &params.config);

        let output = get_output_dir()?.join(file_name);

        Ok(Self {
            id,
            name: info.name,
            cover,
            output,

            params,
            status: Mutex::new(TaskStatus::Pending),
            request_cancel: AtomicBool::default(),
        })
    }

    pub async fn run(&self) -> Result<()> {
        if self.request_cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return Ok(());
        }

        info!("Task #{} '{}' started ({})", self.id, self.name, self.params.path.display());

        *self.status.lock().await = TaskStatus::Loading;

        let mut child = cmd_hidden(std::env::current_exe()?)
            .arg("render")
            .arg(ASSET_PATH.get().unwrap())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        stdin
            .write_all(format!("{}\n", serde_json::to_string(&self.params)?).as_bytes())
            .await?;
        stdin
            .write_all(format!("{}\n", serde_json::to_string(&self.output)?).as_bytes())
            .await?;
        stdin.flush().await?;

        let mut stdout_lines = BufReader::new(stdout).lines();
        let mut stderr_lines = BufReader::new(stderr).lines();
        let mut output_stderr = String::new();
        let mut total_mixing: u64 = 0;
        let mut mixing_count: u64 = 0;
        let mut total_frame: u64 = 0;
        let mut frame_count: u64 = 0;
        let start = Instant::now();
        let mut frame_times = VecDeque::new();
        let mut last_update_fps_sec: u32 = 0;
        let mut last_fps: usize = 0;

        let config = read_config()?;

        loop {
            tokio::select! {
                stderr_result = stderr_lines.next_line() => {
                    let line = stderr_result?;
                    let Some(line) = line else { break };
                    if config.print_stderr {
                        eprintln!("{}", line);
                    }
                    output_stderr.push_str(&line);
                    output_stderr.push('\n');
                },

                stdout_result = stdout_lines.next_line() => {
                    let line = stdout_result?;
                    let Some(line) = line else { break };
                    let Ok(event): Result<IPCEvent, _> = serde_json::from_str(line.trim()) else {
                        continue;
                    };

                    match event {
                        IPCEvent::Loading => {
                            *self.status.lock().await = TaskStatus::Loading;
                        },
                        IPCEvent::StartMixing => {
                            *self.status.lock().await = TaskStatus::Mixing;
                        },
                        IPCEvent::StartMixingSfx(total) => {
                            *self.status.lock().await = TaskStatus::MixingSfx {
                                progress: 0.0,
                            };
                            total_mixing = total;
                        },
                        IPCEvent::Sfx => {
                            mixing_count += 1;
                            *self.status.lock().await = TaskStatus::MixingSfx {
                                progress: mixing_count as f64 / total_mixing as f64,
                            };
                        },
                        IPCEvent::StartRender(total) => {
                            *self.status.lock().await = TaskStatus::Rendering {
                                progress: 0.0,
                                fps: 0,
                                estimate: 0.0,
                            };
                            total_frame = total;
                        },
                        IPCEvent::Frame => {
                            frame_count += 1;
                            let cur = start.elapsed().as_secs_f64();
                            let sec = cur as u32;
                            frame_times.push_back(cur);
                            while frame_times.front().is_some_and(|it| cur - *it > 1.0) {
                                frame_times.pop_front();
                            }
                            if last_update_fps_sec != sec {
                                last_fps = frame_times.len();
                                last_update_fps_sec = sec;
                            }
                            let estimate = total_frame.saturating_sub(frame_count).max(1) as f64 / last_fps as f64;
                            *self.status.lock().await = TaskStatus::Rendering {
                                progress: frame_count as f64 / total_frame as f64,
                                fps: last_fps as u64,
                                estimate,
                            };
                        },
                        IPCEvent::Done(duration) => {
                            info!("Task #{} completed in {}", self.id, format_duration(Duration::from_secs_f64(duration)));
                            child.wait().await?;

                            *self.status.lock().await = TaskStatus::Done {
                                duration,
                                output: output_stderr,
                            };
                            return Ok(());
                        }
                    }
                },

                _ = async {
                    while !self.request_cancel.load(Ordering::Relaxed) {
                        sleep(Duration::from_millis(50)).await;
                    }
                } => {
                    info!("Task #{} cancelled", self.id);
                    child.kill().await?;
                    *self.status.lock().await = TaskStatus::Canceled {
                        output: output_stderr,
                    };
                    return Ok(());
                },
            }
        }

        info!("Task #{} not completed", self.id);
        let status = child.wait().await?;
        if !status.success() {
            *self.status.lock().await = TaskStatus::Failed {
                output: format!(
                    "Child process exited abnormally ({:?})\n{}",
                    status.code().unwrap_or_default(), output_stderr
                ),
            };
            return Ok(());
        }

        Ok(())
    }

    pub fn cancel(&self) {
        self.request_cancel.store(true, Ordering::Relaxed);
    }

    pub async fn to_view(&self) -> TaskView {
        TaskView {
            id: self.id,
            name: self.name.clone(),
            output: self.output.clone(),
            info: self.params.info.clone(),
            config: self.params.config.clone(),
            path: self.params.path.display().to_string(),
            cover: self.cover.path().display().to_string(),
            status: self.status.lock().await.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct TaskView {
    pub id: u32,
    name: String,
    pub output: PathBuf,
    info: ChartInfo,
    config: RenderConfig,
    path: String,
    cover: String,
    status: TaskStatus,
}

pub struct TaskQueue {
    sender: mpsc::UnboundedSender<Arc<Task>>,
    worker: JoinHandle<()>,

    tasks: Mutex<Vec<Arc<Task>>>,
}
impl TaskQueue {
    pub fn new() -> Self {
    let (sender, mut receiver) = mpsc::unbounded_channel::<Arc<Task>>();

    let worker = tokio::spawn(async move {
        let spawn_quene = Arc::new(Semaphore::new(1));
        let mut queue: VecDeque<Arc<Task>> = VecDeque::new();

        loop {
            while let Ok(task) = receiver.try_recv() {
                queue.push_back(task);
            }

            for task in queue.iter() {
                if task.request_cancel.load(std::sync::atomic::Ordering::Relaxed) {
                    let mut status = task.status.lock().await;
                    if !matches!(*status, TaskStatus::Canceled { .. }) {
                        *status = TaskStatus::Canceled { output: String::new() };
                        info!("Task #{} canceled", task.id);
                    }
                }
            }

            if let Some(task) = queue.pop_front() {
                if let Ok(permit) = spawn_quene.clone().try_acquire_owned() {
                    tokio::spawn(async move {
                        if let Err(err) = task.run().await {
                            error!("Task #{} failed: {:?}", task.id, err);
                            let mut status = task.status.lock().await;
                            *status = TaskStatus::Failed { output: format!("{err:?}") };
                        }
                        drop(permit);
                    });
                } else {
                    queue.push_front(task);
                }
            }
        sleep(Duration::from_millis(50)).await;
        }
    });

        Self {
            sender,
            worker,

            tasks: Mutex::default(),
        }
    }

    pub async fn post(&self, params: RenderParams) -> Result<u32> {
        let mut guard = self.tasks.lock().await;
        let id = guard.len() as u32;
        let task = Arc::new(Task::new(id, params).await?);
        guard.push(Arc::clone(&task));
        self.sender.send(task)?;

        Ok(id)
    }

    pub async fn tasks(&self) -> Vec<TaskView> {
        let guard = self.tasks.lock().await;
        let mut result = Vec::with_capacity(guard.capacity());
        for task in guard.iter() {
            result.push(task.to_view().await);
        }
        result.reverse();
        result
    }

    pub async fn cancel(&self, id: u32) {
        self.tasks.lock().await[id as usize].cancel();
    }

    pub async fn remove(&self, id: u32) {
        self.tasks.lock().await.remove(id as usize);
    }

    pub async fn clear(&self) {
        for task in self.tasks.lock().await.drain(..) {
            task.cancel();
        }
    }
}

impl Drop for TaskQueue {
    fn drop(&mut self) {
        self.worker.abort();
    }
}
