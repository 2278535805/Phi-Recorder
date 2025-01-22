use crate::{
    cmd_hidden,
    common::output_dir,
    render::{IPCEvent, RenderParams},
    ASSET_PATH,
};
use anyhow::Result;
use chrono::Local;
use prpr::fs;
use serde::Serialize;
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
    time::Instant,
};
use tempfile::NamedTempFile;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tracing::{error, info};

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
    Canceled,
    Failed {
        error: String,
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
        let config = params.config.to_config();
        let mut cover = NamedTempFile::new()?;
        cover.write_all(&fs.load_file(&info.illustration).await?)?;

        let level: String = info.level.split_whitespace().next().unwrap_or("UK").to_string();
        let safe_name: String = info
            .name
            .chars()
            .filter(|&it| it == '-' || it == '_' || it == ' ' || it.is_alphanumeric())
            .collect();
        let format = if config.hires {"mov"} else {"mp4"};
        let output = output_dir()?.join(format!(
            "{} {safe_name}_{level}.{format}",
            Local::now().format("%Y-%m-%d %H-%M-%S")
        ));

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
        info!("Task #{} started ({})", self.id, self.params.path.display());

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
        let mut total = 0;
        let mut frame_count: u64 = 0;
        let start = Instant::now();
        let mut frame_times = VecDeque::new();
        let mut last_update_fps_sec: u32 = 0;
        let mut last_fps: usize = 0;
        loop {
            let line = lines.next_line().await?;
            let Some(line) = line else { break };
            let Ok(event): Result<IPCEvent, _> = serde_json::from_str(line.trim()) else { continue };
            match event {
                IPCEvent::StartMixing => {
                    *self.status.lock().await = TaskStatus::Mixing;
                }
                IPCEvent::StartRender(total_frame) => {
                    *self.status.lock().await = TaskStatus::Rendering {
                        progress: 0.,
                        fps: 0,
                        estimate: 0.,
                    };
                    total = total_frame;
                }
                IPCEvent::Frame => {
                    frame_count += 1;
                    let cur = start.elapsed().as_secs_f64();
                    let sec = cur as u32;
                    frame_times.push_back(cur);
                    while frame_times.front().is_some_and(|it| cur - *it > 1.) {
                        frame_times.pop_front();
                    }
                    if last_update_fps_sec != sec {
                        last_fps = frame_times.len();
                        last_update_fps_sec = sec;
                    }
                    let estimate =
                        total.saturating_sub(frame_count).max(1) as f64 / last_fps as f64;
                    *self.status.lock().await = TaskStatus::Rendering {
                        progress: frame_count as f64 / total as f64,
                        fps: last_fps as u64,
                        estimate,
                    };
                }
                IPCEvent::Done(duration) => {
                    let output = child.wait_with_output().await?;
                    let stdout = String::from_utf8(output.stdout)
                        .unwrap_or_else(|_| "Invalid output".to_owned());
                    let stderr = String::from_utf8(output.stderr)
                        .unwrap_or_else(|_| "Invalid output".to_owned());
                    *self.status.lock().await = TaskStatus::Done {
                        duration,
                        //output: format!("[STDOUT]\n{stdout}\n\n[STDERR]\n{stderr}"),
                        output: format!("{stdout}\n{stderr}"),
                    };
                    return Ok(());
                }
            }
            if self.request_cancel.load(Ordering::Relaxed) {
                child.kill().await?;
                *self.status.lock().await = TaskStatus::Canceled;
                return Ok(());
            }
        }

        let output = child.wait_with_output().await?;
        if !output.status.success() {
            *self.status.lock().await = TaskStatus::Failed {
                error: format!(
                    "Child process exited abnormally ({:?})\n{}\n{}",
                    output.status.code().unwrap_or_default(),
                    String::from_utf8(output.stdout)?,
                    String::from_utf8(output.stderr)?
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
            path: self.params.path.display().to_string(),
            cover: self.cover.path().display().to_string(),
            status: self.status.lock().await.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct TaskView {
    id: u32,
    name: String,
    output: PathBuf,
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
        let task = tokio::spawn(async move {
            loop {
                let Ok(task) = receiver.try_recv() else {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    continue
                };
                if let Err(err) = task.run().await {
                    error!("Failed to render: {err:?}");
                    *task.status.lock().await = TaskStatus::Failed {
                        error: format!("{err:?}"),
                    };
                }
            }
        });

        Self {
            sender,
            worker: task,

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
}

impl Drop for TaskQueue {
    fn drop(&mut self) {
        self.worker.abort();
    }
}
