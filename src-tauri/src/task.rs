use crate::{
    cmd_hidden, common::{output_dir, read_config}, ipc::IPCEvent, render::{RenderConfig, RenderParams}, ASSET_PATH
};
use anyhow::Result;
use chrono::Local;
use phire::{fs, info::ChartInfo};
use serde::Serialize;
use tracing::{error, info};
use std::{
    collections::VecDeque,
    io::Write,
    ops::DerefMut,
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

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum TaskStatus {
    Pending,
    Loading,
    Mixing,
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
        let info = fs::load_info(fs.deref_mut()).await?;
        let mut cover = NamedTempFile::new()?;
        cover.write_all(&fs.load_file(&info.illustration).await?)?;

        let level: String = info
            .level
            .split_whitespace()
            .next()
            .unwrap_or("UK")
            .to_string();
        let safe_name: String = info
            .name
            .chars()
            .filter(|&it| it == '-' || it == '_' || it == ' ' || it.is_alphanumeric())
            .collect();
        let format = if params.config.hires { "mov" } else { "mp4" };
        let file_name = if params.config.simple_file_name {
            let safe_name2: String = info
                .composer
                .chars()
                .filter(|&it| it == '-' || it == '_' || it.is_alphanumeric())
                .collect();
            format!("{safe_name}.{safe_name2}_{level}.{format}",)
        } else {
            format!(
                "{} {safe_name}_{level}.{format}",
                Local::now().format("%Y-%m-%d %H-%M-%S")
            )
        };

        let output = if let Some(set_output_dir) = read_config()?.output_dir {
            set_output_dir.join(file_name)
        } else {
            output_dir()?.join(file_name)
        };

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

        stdin
            .write_all(format!("{}\n", serde_json::to_string(&self.params)?).as_bytes())
            .await?;
        stdin
            .write_all(format!("{}\n", serde_json::to_string(&self.output)?).as_bytes())
            .await?;
        stdin.flush().await?;

        let mut lines = BufReader::new(stdout).lines();
        let mut total: u64 = 0;
        let mut frame_count: u64 = 0;
        let start = Instant::now();
        let mut frame_times = VecDeque::new();
        let mut last_update_fps_sec: u32 = 0;
        let mut last_fps: usize = 0;

        loop {
            tokio::select! {
                _ = async {
                    while !self.request_cancel.load(Ordering::Relaxed) {
                        sleep(Duration::from_millis(50)).await;
                    }
                } => {
                    info!("Task #{} cancelled", self.id);
                    child.kill().await?;
                    let output = child.wait_with_output().await?;
                    *self.status.lock().await = TaskStatus::Canceled {
                        output: format!(
                            "{}\n{}",
                            String::from_utf8(output.stdout)
                                .unwrap_or_else(|error| format!("Invalid output: {}", error.to_string())),
                            String::from_utf8(output.stderr)
                                .unwrap_or_else(|error| format!("Invalid output: {}", error.to_string()))
                        ),
                    };
                    return Ok(());
                },

                line_result = lines.next_line() => {
                    let line = line_result?;
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
                        IPCEvent::StartRender(total_frame) => {
                            *self.status.lock().await = TaskStatus::Rendering {
                                progress: 0.0,
                                fps: 0,
                                estimate: 0.0,
                            };
                            total = total_frame;
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
                            let estimate = total.saturating_sub(frame_count).max(1) as f64 / last_fps as f64;
                            *self.status.lock().await = TaskStatus::Rendering {
                                progress: frame_count as f64 / total as f64,
                                fps: last_fps as u64,
                                estimate,
                            };
                        },
                        IPCEvent::Done(duration) => {
                            info!("Task #{} completed", self.id);
                            let output = child.wait_with_output().await?;
                            *self.status.lock().await = TaskStatus::Done {
                                duration,
                                output: format!(
                                    "{}\n{}",
                                    String::from_utf8(output.stdout)
                                        .unwrap_or_else(|error| format!("Invalid output: {}", error.to_string())),
                                    String::from_utf8(output.stderr)
                                        .unwrap_or_else(|error| format!("Invalid output: {}", error.to_string()))
                                ),
                            };
                            return Ok(());
                        }
                    }
                }
            }
        }

        info!("Task #{} not completed", self.id);
        let output = child.wait_with_output().await?;
        if !output.status.success() {
            *self.status.lock().await = TaskStatus::Failed {
                output: format!(
                    "Child process exited abnormally ({:?})\n{}\n{}",
                    output.status.code().unwrap_or_default(),
                    String::from_utf8(output.stdout)
                        .unwrap_or_else(|error| format!("Invalid output: {}", error.to_string())),
                    String::from_utf8(output.stderr)
                        .unwrap_or_else(|error| format!("Invalid output: {}", error.to_string()))
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
