// Prevents additional console window on Windows in release, DO NOT REMOVE!!
phire::tl_file!("render");

use crate::{
    common::{get_output_dir, parse_args, read_config, test_output_dir},
    ipc::{client::*, IPCEvent},
    task::generate_filename,
    ASSET_PATH
};
use anyhow::{bail, Context, Result};
use macroquad::{miniquad::gl::*, prelude::*};
use num_complex::Complex;
use ndarray::{s, Array1};
use phire::{
    Main, config::{ChallengeModeColor, Config, Mods}, core::{HitSound, MSRenderTarget, Note, ResourcePack, internal_id}, ext::{BLACK_TEXTURE, NotNanExt, SafeTexture}, fs::{self, FileSystem}, info::ChartInfo, scene::{BasicPlayer, EndingScene, GameMode, GameScene, LoadingScene, game::WAIT_TIME}, time::TimeManager, ui::{FontArc, TextPainter}
};
use rustc_hash::FxHashMap;
use rayon::prelude::*;
use sasa::AudioClip;
use realfft::RealFftPlanner;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    io::{BufRead, Write},
    ops::DerefMut,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    rc::Rc,
    sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex},
    time::{Duration, Instant},
};
use std::{ffi::OsStr, fmt::Write as _};
use tempfile::NamedTempFile;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct RenderConfig {
    pub resolution: (u32, u32),
    pub ending_length: f64,
    pub render_loading: bool,
    pub hires: bool,
    pub chart_debug_line: f32,
    pub chart_debug_note: f32,
    pub chart_ratio: f32,
    pub all_good: bool,
    pub all_bad: bool,
    pub fps: u32,
    pub hardware_accel: bool,
    pub hevc: bool,
    pub mpeg4: bool,
    pub custom_encoder: Option<String>,
    pub dynamic_bitrate_control: bool,
    pub bitrate: String,

    pub aggressive_chart: bool,
    pub aggressive_note: bool,
    pub aggressive_particle: bool,
    pub challenge_color: ChallengeModeColor,
    pub challenge_rank: u32,
    pub note_scale: f32,
    //pub offset: f32,
    pub particle: bool,
    pub player_avatar: Option<String>,
    pub player_name: String,
    pub player_rks: f32,
    pub sample_count: u32,
    pub fxaa: bool,
    pub res_pack_path: Option<String>,
    pub speed: f32,
    pub volume_music: f32,
    pub volume_sfx: f32,
    pub force_limit: bool,
    pub limit_threshold: f32,
    pub loudness_equalization: bool,
    pub audio_mix_mode: AudioMixMode,
    pub watermark: String,
    pub roman: bool,
    pub chinese: bool,
    pub combo: String,
    pub difficulty: String,
    pub judge_offset: f64,
    pub file_name_format: String,

    pub render_line: bool,
    pub render_line_extra: bool,
    pub render_note: bool,
    pub render_double_hint: bool,
    pub render_ui_pause: bool,
    pub render_ui_name: bool,
    pub render_ui_level: bool,
    pub render_ui_score: bool,
    pub render_ui_combo: bool,
    pub render_ui_bar: bool,
    pub render_bg: bool,
    pub render_bg_dim: bool,
    pub preserve_framebuffer: bool,
    pub render_extra: bool,
    pub bg_blurriness: f32,

    pub max_particles: usize,
    pub play_start_time: f64,
    pub play_end_time: Option<f64>,

    pub fade: f32,
    pub alpha_tint: bool,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AudioMixMode {
    Traditional,
    #[default]
    Optimized,
    Fft,
}

impl AudioMixMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Traditional => "traditional",
            Self::Optimized => "optimized",
            Self::Fft => "fft",
        }
    }
}

impl RenderConfig {
    pub fn to_config(&self) -> Config {
        Config {
            aggressive_chart: self.aggressive_chart,
            aggressive_note: self.aggressive_note,
            aggressive_particle: self.aggressive_particle,
            challenge_color: self.challenge_color.clone(),
            challenge_rank: self.challenge_rank,
            enter_animation: self.render_loading,
            fxaa: self.fxaa,
            note_scale: self.note_scale,
            //offset: self.offset,
            particle: self.particle,
            player_name: self.player_name.clone(),
            player_rks: self.player_rks,
            sample_count: self.sample_count,
            res_pack_path: self.res_pack_path.clone(),
            speed: self.speed,
            volume_music: self.volume_music,
            volume_sfx: self.volume_sfx,
            chart_debug_line: self.chart_debug_line,
            chart_debug_note: self.chart_debug_note,
            chart_ratio: self.chart_ratio,
            all_good: self.all_good,
            all_bad: self.all_bad,
            watermark: self.watermark.clone(),
            roman: self.roman,
            chinese: self.chinese,
            combo: self.combo.clone(),
            difficulty: self.difficulty.clone(),
            judge_offset: self.judge_offset,

            render_line: self.render_line,
            render_line_extra: self.render_line_extra,
            render_note: self.render_note,
            render_double_hint: self.render_double_hint,
            render_ui_pause: self.render_ui_pause,
            render_ui_name: self.render_ui_name,
            render_ui_level: self.render_ui_level,
            render_ui_score: self.render_ui_score,
            render_ui_combo: self.render_ui_combo,
            render_ui_bar: self.render_ui_bar,
            render_bg: self.render_bg,
            render_bg_dim: self.render_bg_dim,
            preserve_framebuffer: self.preserve_framebuffer,
            render_extra: self.render_extra,
            bg_blurriness: self.bg_blurriness,

            max_particles: self.max_particles,
            play_start_time: self.play_start_time,
            play_end_time: self.play_end_time,

            fade: self.fade,
            alpha_tint: self.alpha_tint,
            ..Default::default()
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            ending_length: 0.0,
            render_loading: false,
            hires: false,
            fps: 60,
            hardware_accel: true,
            hevc: false,
            mpeg4: false,
            custom_encoder: None,
            dynamic_bitrate_control: true,
            bitrate: "28".to_string(),
            aggressive_chart: true,
            aggressive_note: false,
            aggressive_particle: false,
            challenge_color: ChallengeModeColor::Rainbow,
            challenge_rank: 3,
            fxaa: false,
            note_scale: 1.0,
            particle: true,
            player_name: "HLMC".to_string(),
            player_rks: 16.0,
            sample_count: 8,
            res_pack_path: None,
            speed: 1.0,
            volume_music: 0.5,
            volume_sfx: 0.4,
            force_limit: true,
            limit_threshold: 0.5,
            loudness_equalization: false,
            audio_mix_mode: AudioMixMode::Fft,
            chart_debug_line: 0.0,
            chart_debug_note: 0.0,
            chart_ratio: 1.0,
            all_good: false,
            all_bad: false,
            watermark: "".to_string(),
            roman: false,
            chinese: false,
            combo: "AUTOPLAY".to_string(),
            difficulty: "".to_string(),
            player_avatar: None,
            judge_offset: 0.,
            file_name_format: "%date% %time% %info.name%_%level_prefix%".to_string(),

            render_line: true,
            render_line_extra: true,
            render_note: true,
            render_double_hint: true,
            render_ui_pause: true,
            render_ui_name: true,
            render_ui_level: true,
            render_ui_score: true,
            render_ui_combo: true,
            render_ui_bar: true,
            render_bg: true,
            render_bg_dim: true,
            preserve_framebuffer: false,
            render_extra: true,
            bg_blurriness: 80.,

            max_particles: 5000,
            play_start_time: 0.0,
            play_end_time: None,

            fade: 0.0,
            alpha_tint: false,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderParams {
    pub path: PathBuf,
    pub info: ChartInfo,
    pub config: RenderConfig,
}

pub async fn build_player(config: &RenderConfig) -> Result<BasicPlayer> {
    Ok(BasicPlayer {
        avatar: if let Some(path) = &config.player_avatar {
            Some(
                Texture2D::from_file_with_format(
                    &tokio::fs::read(path)
                        .await
                        .with_context(|| tl!("load-avatar-failed"))?,
                    None,
                )
                .into(),
            )
        } else {
            None
        },
        id: 0,
        rks: config.player_rks,
    })
}

fn cmd_hidden(program: impl AsRef<OsStr>) -> Command {
    let cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = cmd;
        cmd.creation_flags(0x08000000);
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    cmd
}

pub fn test_ffmpeg(path: impl AsRef<OsStr>) -> bool {
    matches!(cmd_hidden(path).arg("-version").output(), Ok(_))
}

pub fn find_ffmpeg() -> Result<Option<String>> {
    if let Some(ffmpeg_path) = read_config()?.ffmpeg_path {
        if test_ffmpeg(&ffmpeg_path) {
            return Ok(Some(ffmpeg_path));
        }
    }
    if test_ffmpeg("ffmpeg") {
        return Ok(Some("ffmpeg".to_owned()));
    }
    let exe_dir = std::env::current_exe()?.parent().unwrap().to_owned();
    let ffmpeg = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    let ffmpeg = exe_dir.join(ffmpeg);
    Ok(if test_ffmpeg(&ffmpeg) {
        Some(ffmpeg.display().to_string())
    } else {
        None
    })
}

pub const ENCODER_LIST_HEVC: [&str; 4] = ["hevc_nvenc", "hevc_qsv", "hevc_amf", "hevc_vaapi"];
pub const ENCODER_LIST_AVC: [&str; 4] = ["h264_nvenc", "h264_qsv", "h264_amf", "h264_vaapi"];

pub fn get_encoder(
    ffmpeg: &String,
    config: &RenderConfig,
    encoder_list: [&str; 4],
    use_global_config: bool,
) -> Option<String> {
    if let Some(custom_encoder) = &config.custom_encoder {
        return Some(custom_encoder.to_string());
    };

    if config.mpeg4 {
        return Some("mpeg4".to_string());
    };

    if !config.hardware_accel {
        if config.hevc {
            return Some("libx265".to_string());
        } else {
            return Some("libx264".to_string());
        }
    }

    if use_global_config {
        let global_config = read_config().unwrap_or_default();
        if let Some(encoder_avc) = global_config.encoder_avc {
            if !config.hevc && !config.mpeg4 {
                return Some(encoder_avc);
            }
        }
        if let Some(encoder_hevc) = global_config.encoder_hevc {
            if config.hevc && !config.mpeg4 {
                return Some(encoder_hevc);
            }
        }
    }

    for encoder in encoder_list {
        if test_encoder(ffmpeg, encoder) {
            return Some(encoder.to_string());
        } else {
            warn!("Encoder {} not supported", encoder);
        }
    }

    None
}

pub fn test_encoder(ffmpeg: &String, encoder: &str) -> bool {
    eprintln!("Testing encoder: {}", encoder);
    let output = Command::new(ffmpeg)
        .args(["-f", "lavfi", "-i", "testsrc=size=1920x1080:rate=5:duration=1", "-pix_fmt", "yuv420p", "-c:v", encoder, "-f", "null", "-"])
        .args(["-loglevel", "warning"])
        // .arg("-hide_banner")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| tl!("run-ffmpeg-failed"))
        .expect("failed test encoder");
    output.status.success()
}

fn round_to_step(v: f64, step: f64) -> f64 {
    (v / step).round() * step
}

struct PreparedSfx {
    positions: Vec<usize>,
    spectrum: Vec<Complex<f32>>,
}

struct SfxFftWorker {
    forward: Arc<dyn realfft::RealToComplex<f32>>,
    inverse: Arc<dyn realfft::ComplexToReal<f32>>,
    impulse: Vec<f32>,
    impulse_fft: Vec<Complex<f32>>,
    total_fft: Vec<Complex<f32>>,
    mixed: Vec<f32>,
}

impl SfxFftWorker {
    fn new(fft_size: usize) -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        Self {
            forward: planner.plan_fft_forward(fft_size),
            inverse: planner.plan_fft_inverse(fft_size),
            impulse: vec![0.0; fft_size],
            impulse_fft: vec![Complex::new(0.0, 0.0); fft_size / 2 + 1],
            total_fft: vec![Complex::new(0.0, 0.0); fft_size / 2 + 1],
            mixed: vec![0.0; fft_size],
        }
    }

    fn mix_block(
        &mut self,
        output: &mut [f32],
        block_start: usize,
        block_len: usize,
        overlap: usize,
        groups: &[PreparedSfx],
    ) -> Result<()> {
        let input_start = block_start as isize - overlap as isize;
        let input_end = input_start + self.impulse.len() as isize;

        self.total_fft.fill(Complex::new(0.0, 0.0));
        for group in groups {
            self.impulse.fill(0.0);
            let first = group.positions.partition_point(|&position| (position as isize) < input_start);
            let last = group.positions.partition_point(|&position| (position as isize) < input_end);
            for &position in &group.positions[first..last] {
                let impulse_position = position as isize - input_start;
                if impulse_position >= 0 {
                    self.impulse[impulse_position as usize] += 1.0;
                }
            }

            self.forward.process(&mut self.impulse, &mut self.impulse_fft)?;
            for (total, (impulse, clip)) in self
                .total_fft
                .iter_mut()
                .zip(self.impulse_fft.iter().zip(&group.spectrum))
            {
                *total += *impulse * *clip;
            }
        }

        self.inverse.process(&mut self.total_fft, &mut self.mixed)?;
        let scale = 1.0 / self.impulse.len() as f32;
        let valid_len = output.len().min(block_len);
        for (target, &value) in output[..valid_len]
            .iter_mut()
            .zip(&self.mixed[overlap..overlap + valid_len])
        {
            *target = value * scale;
        }
        Ok(())
    }
}

fn mix_sfx_fft(output: &mut Array1<f32>, groups: &mut [(&Array1<f32>, Vec<usize>)], ipc: bool) -> Result<(usize, usize)> {
    if groups.iter().all(|(clip, positions)| clip.is_empty() || positions.is_empty()) || output.is_empty() {
        return Ok((0, 0));
    }

    let max_clip_len = groups.iter().filter(|(clip, positions)| !clip.is_empty() && !positions.is_empty()).map(|(clip, _)| clip.len()).max().unwrap();
    const TARGET_BLOCK_LEN: usize = 1 << 17;
    let fft_size = (max_clip_len + TARGET_BLOCK_LEN).next_power_of_two();
    let overlap = max_clip_len - 1;
    let block_len = ((fft_size - overlap) / 2) * 2;
    let block_count = output.len().div_ceil(block_len);
    if ipc {
        send(IPCEvent::MixingSfx(block_count as u64 + 1));
    }

    let mut planner = RealFftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(fft_size);
    let mut prepared = Vec::with_capacity(groups.len());
    for (clip, positions) in groups.iter_mut() {
        if clip.is_empty() || positions.is_empty() {
            continue;
        }
        positions.sort_unstable();
        let mut input = vec![0.0; fft_size];
        input[..clip.len()].copy_from_slice(clip.as_slice().unwrap());
        let mut spectrum = vec![Complex::new(0.0, 0.0); fft_size / 2 + 1];
        forward.process(&mut input, &mut spectrum)?;
        prepared.push(PreparedSfx { positions: std::mem::take(positions), spectrum });
    }

    let output_slice = output.as_slice_mut().unwrap();
    if ipc {
        send(IPCEvent::Sfx(1));
    }
    let completed = Mutex::new(1);
    output_slice
        .par_chunks_mut(block_len)
        .enumerate()
        .try_for_each_init(
            || SfxFftWorker::new(fft_size),
            |worker, (index, block)| {
                worker.mix_block(block, index * block_len, block_len, overlap, &prepared)?;
                if ipc {
                    let mut completed = completed.lock().unwrap();
                    *completed += 1;
                    send(IPCEvent::Sfx(*completed));
                }
                Ok::<(), anyhow::Error>(())
            },
        )?;

    Ok((fft_size, block_len))
}

pub async fn generate_resource(is_cli: bool, generate_output: bool) -> Result<(Box<dyn FileSystem + Send + Sync>, PathBuf, RenderConfig, ChartInfo)> {
    if is_cli {
        let (args_input, args_output, args_config, args_info) = parse_args(std::env::args().collect());

        let config: RenderConfig = if let Some(config) = &args_config {
            match serde_json::from_str(config) {
                Ok(config_json) => {
                    eprintln!("Using config from json");
                    config_json
                }
                Err(error) => {
                    eprintln!("{}", error);
                    eprintln!("Failed to parse json. Using config from toml file");
                    toml::from_str(&std::fs::read_to_string(config)?)?
                }
            }
        } else {
            eprintln!("Using config from config.toml");
            toml::from_str(&std::fs::read_to_string(std::env::current_exe()?.parent().unwrap().join("config.toml"))?)?
        };

        let path = args_input.unwrap();

        let mut fs = fs::fs_from_file(path.as_ref())?;

        let info = if let Some(info) = args_info {
            serde_json::from_str(&info)?
        } else {
                fs::load_info(fs.deref_mut()).await?
            };

        let output_path = if generate_output {
            let file_name = generate_filename(&info, &config);

            let output_path = if let Some(output_string) = args_output {
                let output_dir = PathBuf::from(output_string);
                if output_dir.extension().is_some() {
                    output_dir
                } else {
                    test_output_dir(output_dir.clone())?;
                    output_dir.join(file_name)
                }
            } else {
                get_output_dir()?.join(file_name) 
            };
            eprintln!("output file: {:?}", output_path);
            output_path
        } else {
            PathBuf::new()
        };

        Ok((fs, output_path, config, info))
    } else {
        let mut stdin = std::io::stdin().lock();

        let mut line = String::new();
        stdin.read_line(&mut line)?;
        let params: RenderParams = serde_json::from_str(line.trim())?;
        let path = params.path;
        
        let output_path = if generate_output {
            line.clear();
            stdin.read_line(&mut line)?;
            serde_json::from_str::<PathBuf>(line.trim())?
        } else {
            PathBuf::new()
        };

        let fs = fs::fs_from_file(&path)?;

        let config = params.config;
        let info = params.info;

        Ok((fs, output_path, config, info))
    }
}

pub async fn main(cmd: bool) -> Result<()> {
    let loading_time = Instant::now();
    let (mut fs, output_path, mut config, info) = generate_resource(cmd, true).await?;

    set_pc_assets_folder(ASSET_PATH.get().unwrap().to_str().unwrap());
    let ipc = !cmd;
    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;
    let mut painter = TextPainter::new(font);
    let volume_music = std::mem::take(&mut config.volume_music);
    let volume_sfx = std::mem::take(&mut config.volume_sfx);
    let mut prpr_config = config.to_config();
    prpr_config.mods = Mods::AUTOPLAY;
    let Some(ffmpeg) = find_ffmpeg()? else {
        bail!("FFmpeg not found")
    };

    let pause_requested = Arc::new(AtomicBool::new(false));
    if !cmd {
        let pause_requested = Arc::clone(&pause_requested);
        let render_thread = std::thread::current();
        std::thread::spawn(move || {
            let stdin = std::io::stdin();
            let mut line = String::new();
            loop {
                line.clear();
                match stdin.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => match line.trim() {
                        "pause" => pause_requested.store(true, Ordering::SeqCst),
                        "resume" => {
                            pause_requested.store(false, Ordering::SeqCst);
                            render_thread.unpark();
                        }
                        _ => {}
                    },
                    Err(_) => break,
                }
            }
        });
    }

    let (chart, format) = GameScene::load_chart(fs.deref_mut(), &info, &prpr_config)
        .await
        .with_context(|| tl!("load-chart-failed"))?;
    let res_pack = ResourcePack::from_path(config.res_pack_path.as_ref())
        .await
        .context("Failed to load resource pack")?;
    let music: Result<_> = async { AudioClip::new(fs.load_file(&info.music).await?) }.await;
    let music = music.with_context(|| tl!("load-music-failed"))?;
    let music_length = music.length();
    let music_sample_rate = music.sample_rate();
    let ending_music = res_pack.endings[0].clone();
    let ending_music_sample_rate = ending_music.sample_rate();
    let sfx_click = res_pack.sfx_click;
    let sfx_drag = res_pack.sfx_drag;
    let sfx_flick = res_pack.sfx_flick;

    let sample_rate = 48000;
    let sample_rate_f64 = sample_rate as f64;
    let sfx_protect_time = if let Some(sfx_longest) = chart.hitsounds.values().max_by_key(|v| v.length().not_nan()) {
        sfx_longest.length()
    } else {
        sfx_drag.length()
    };

    fn check_sample_rate(expected: u32, actual: u32, name: &str) -> Result<()> {
        if expected != actual {
            bail!(
                tl!("match-sample-rate-failed", "expected" => expected, "name" => name, "actual" => actual)
            );
        } else {
            Ok(())
        }
    }
    check_sample_rate(sample_rate, ending_music.sample_rate(), "ending_music")?;
    check_sample_rate(sample_rate, sfx_click.sample_rate(), "sfx_click")?;
    check_sample_rate(sample_rate, sfx_drag.sample_rate(), "sfx_drag")?;
    check_sample_rate(sample_rate, sfx_flick.sample_rate(), "sfx_flick")?;

    let music = Array1::from_vec(music.to_vec());
    let ending_music = Array1::from_vec(ending_music.to_vec());
    let sfx_click = Array1::from_vec(sfx_click.to_vec());
    let sfx_drag = Array1::from_vec(sfx_drag.to_vec());
    let sfx_flick = Array1::from_vec(sfx_flick.to_vec());

    let mut gl = unsafe { get_internal_gl() };

    let fps = config.fps;
    let offset = chart.offset + info.offset;
    let speed = config.speed as f64;
    let speed_time_ratio = 1.0 / speed;

    let before_time: f64 = if config.render_loading {
        LoadingScene::TOTAL_TIME + GameScene::BEFORE_DURATION * speed_time_ratio
    } else {
        0.0
    };
    let before_time_music: f64 = if config.render_loading {
        LoadingScene::TOTAL_TIME * speed + GameScene::BEFORE_DURATION
    } else {
        0.0
    };

    let chart_length = before_time + config.play_end_time.unwrap_or(music_length).min(music_length) * speed_time_ratio - config.play_start_time * speed_time_ratio - offset + WAIT_TIME * speed_time_ratio;
    let chart_length_music = before_time_music + config.play_end_time.unwrap_or(music_length).min(music_length) - config.play_start_time - offset + WAIT_TIME;
    let chart_length_sfx = config.play_end_time.unwrap_or(music_length).min(music_length) - config.play_start_time - offset + WAIT_TIME;
    let video_length = chart_length + config.ending_length;
    let video_length_music = chart_length_music + config.ending_length; // chart_length needs to be divided by speed, but music needs to be rendered at the original speed, which is changed by ffmpeg
    let video_frames = (video_length * fps as f64).ceil() as u64;

    let encoder_list = if config.hevc {
        ENCODER_LIST_HEVC
    } else {
        ENCODER_LIST_AVC
    };

    let ffmpeg_encoder =
        if let Some(ffmpeg_encoder) = get_encoder(&ffmpeg, &config, encoder_list, true) {
            ffmpeg_encoder
        } else {
            bail!(tl!("no-hwacc"))
        };

    eprintln!("Encoder: {}", ffmpeg_encoder);

    eprintln!("Loading Time: {:.2?}", loading_time.elapsed());
    eprintln!("video length: {:.2}s frame: {}", video_length, video_frames);

    let render_start_time = Instant::now();

    if ipc {
        send(IPCEvent::Mixing);
    }

    let output_music_len = (video_length_music * music_sample_rate as f64).ceil() as usize * 2;
    let output_sfx_len = ((video_length + sfx_protect_time) * sample_rate_f64).ceil() as usize * 2;
    let output_ending_music_delay = chart_length + GameScene::WAIT_AFTER_TIME * speed_time_ratio + EndingScene::BPM_WAIT_TIME;
    let output_ending_music_len = ((video_length - output_ending_music_delay).max(0.) * sample_rate_f64).ceil() as usize * 2;
    let output_ending_music_delay_string = output_ending_music_delay * 1000.;
    let output_ending_music_delay_string = format!("{}|{}", output_ending_music_delay_string, output_ending_music_delay_string);

    let mut output_music = Array1::from_vec(vec![0.0_f32; output_music_len]);
    let mut output_sfx = Array1::from_vec(vec![0.0_f32; output_sfx_len]);
    let mut output_ending_music = Array1::from_vec(vec![0.0_f32; output_ending_music_len]);

    let mut place_sfx = |pos: f64, clip: &Array1<f32>| {
        let position = (pos * sample_rate_f64).ceil() as usize * 2;
        let len = clip.len();
        let end = position + len;
        if end > output_sfx_len {
            return;
        }
        let mut slice = output_sfx.slice_mut(s![position..end]);
        slice += clip;
    };

    if volume_music != 0.0 {
        let music_time = Instant::now();
        let pos = (before_time - offset.min(0.)) * speed;
        let position_wrtie = (pos * music_sample_rate as f64).ceil() as usize * 2;
        let position_read = ((offset.max(0.) + config.play_start_time) * music_sample_rate as f64).ceil() as usize * 2;
        let music_len = (chart_length_music * music_sample_rate as f64).ceil() as usize * 2;
        let len = (music.len() - position_read).min(output_music_len - position_wrtie).min(music_len - position_wrtie);
        let clip = music.slice(s![position_read..position_read + len]);
        let mut slice = output_music.slice_mut(s![position_wrtie..position_wrtie + len]);
        slice += &clip;
        eprintln!("Process Music Time: {:.2?}", music_time.elapsed());
    }

    type HitSoundMap = FxHashMap<String, Array1<f32>>;
    let mut extra_sfxs: HitSoundMap = HitSoundMap::with_capacity_and_hasher(16, Default::default());

    chart.hitsounds.iter().for_each(|(name, clip)| {
        extra_sfxs.insert(name.clone(), Array1::from_vec(clip.to_vec()));
    });

    let get_hitsound = |note: &Note| match &note.hitsound {
        HitSound::None => None,
        HitSound::Click => Some(&sfx_click),
        HitSound::Flick => Some(&sfx_flick),
        HitSound::Drag => Some(&sfx_drag),
        HitSound::Custom(s) => extra_sfxs.get(s),
    };

    if volume_sfx != 0.0 {
        let sfx_time = Instant::now();
        let judge_offset = config.judge_offset;
        let sfx_start_time = config.play_start_time - config.judge_offset;
        let sfx_end_time = sfx_start_time + chart_length_sfx;
        let mut sfx_list: Vec<(f64, &Array1<f32>)> = Vec::with_capacity(chart.lines.iter().map(|line| line.notes.len()).sum::<usize>());

        if config.audio_mix_mode == AudioMixMode::Optimized {
            chart.lines.iter().flat_map(|line| &line.notes).filter(|note| !note.fake && note.time > sfx_start_time && note.time < sfx_end_time).for_each(|note| {
                if let Some(sfx) = get_hitsound(&note) {
                    let pos = round_to_step(before_time + note.time * speed_time_ratio + judge_offset - config.play_start_time * speed_time_ratio, 0.005);
                    sfx_list.push((pos, sfx));
                }
            });
            let len = sfx_list.len();

            sfx_list.sort_unstable_by(|(time_a, sfx_a), (time_b, sfx_b)| {
                time_a.total_cmp(time_b).then_with(|| sfx_a.as_ptr().cmp(&sfx_b.as_ptr()))
            });

            let mut kept_sfx_list = Vec::with_capacity(len);
            let mut last_arr: Option<&Array1<f32>> = None;
            let mut last_t = 0.0;
            let mut count = 0;

            for &(pos, clip) in &sfx_list {
                let is_new_group = match last_arr {
                    None => true,
                    Some(prev) => {
                        !std::ptr::eq(prev, clip) || pos != last_t
                    }
                };

                if is_new_group {
                    last_arr = Some(clip);
                    last_t = pos;
                    count = 1;
                    kept_sfx_list.push((pos, clip));
                } else {
                    if count < 3 {
                        kept_sfx_list.push((pos, clip));
                        count += 1;
                    }
                }
            }
            drop(sfx_list);

            let num = kept_sfx_list.len();
            if ipc {
                send(IPCEvent::MixingSfx(num as u64));
            }
            let mut last_sfx_progress = Instant::now();
            for (index, (pos, sfx)) in kept_sfx_list.into_iter().enumerate() {
                place_sfx(pos, sfx);
                let completed = index as u64 + 1;
                if ipc && (last_sfx_progress.elapsed() >= Duration::from_millis(350) || completed == num as u64) {
                    send(IPCEvent::Sfx(completed));
                    last_sfx_progress = Instant::now();
                }
            }

            let elapsed = sfx_time.elapsed();
            eprintln!("Process Hit Effects Time: {:.2?} Equivalent Speed: {:.2} notes/sec Speed: {:.2} notes/sec", elapsed, len as f32 / elapsed.as_secs_f32(), num as f32 / elapsed.as_secs_f32())
        } else if config.audio_mix_mode == AudioMixMode::Fft {
            let mut groups: Vec<(&Array1<f32>, Vec<usize>)> = Vec::new();
            chart.lines.iter().flat_map(|line| &line.notes).filter(|note| !note.fake && note.time > sfx_start_time && note.time < sfx_end_time).for_each(|note| {
                if let Some(sfx) = get_hitsound(&note) {
                    let position = (before_time + note.time * speed_time_ratio + judge_offset - config.play_start_time * speed_time_ratio) * sample_rate_f64;
                    let position = position.ceil() as usize * 2;
                    if position.checked_add(sfx.len()).is_some_and(|end| end <= output_sfx_len) {
                        if let Some((_, positions)) = groups.iter_mut().find(|(clip, _)| std::ptr::eq(*clip, sfx)) {
                            positions.push(position);
                        } else {
                            groups.push((sfx, vec![position]));
                        }
                    }
                }
            });

            let (fft_size, block_len) = mix_sfx_fft(&mut output_sfx, &mut groups, ipc)?;
            eprintln!("Process Hit Effects FFT Time: {:.2?} Groups: {} FFT size: {} Block size: {}", sfx_time.elapsed(), groups.len(), fft_size, block_len);
        } else {
            chart.lines.iter().flat_map(|line| &line.notes).filter(|note| !note.fake && note.time > sfx_start_time && note.time < sfx_end_time).for_each(|note| {
                if let Some(sfx) = get_hitsound(&note) {
                    sfx_list.push((before_time + note.time * speed_time_ratio + judge_offset - config.play_start_time * speed_time_ratio, sfx));
                }
            });
            let num = sfx_list.len();
            if ipc {
                send(IPCEvent::MixingSfx(num as u64));
            }
            let mut last_sfx_progress = Instant::now();
            for (index, (pos, sfx)) in sfx_list.into_iter().enumerate() {
                place_sfx(pos, sfx);
                let completed = index as u64 + 1;
                if ipc && (last_sfx_progress.elapsed() >= Duration::from_millis(350) || completed == num as u64) {
                    send(IPCEvent::Sfx(completed));
                    last_sfx_progress = Instant::now();
                }
            }

            let elapsed = sfx_time.elapsed();
            eprintln!("Process Hit Effects Time: {:.2?} Speed: {:.2} notes/sec", elapsed, num as f32 / elapsed.as_secs_f32())
        }
    }

    if volume_music != 0.0 && output_ending_music_len > 0 {
        let ending_time = Instant::now();
        let mut position_wrtie = 0;
        while position_wrtie < output_ending_music_len {
            let len = (ending_music.len()).min(output_ending_music_len - position_wrtie);
            let clip = ending_music.slice(s![..len]);
            let mut slice = output_ending_music.slice_mut(s![position_wrtie..position_wrtie + len]);
            slice += &clip;
            position_wrtie += len;
        }
        eprintln!("Process Ending Music Time: {:.2?}", ending_time.elapsed());
    }

    if ipc {
        send(IPCEvent::Mixing);
    }
    let output_music_temp = NamedTempFile::new()?;
    let output_sfx_temp = NamedTempFile::new()?;
    let output_ending_temp = NamedTempFile::new()?;

    {
        let output_audio_time = Instant::now();

        let output_audio = |output: &Path, sample_rate: u32, samples: ndarray::Array1<f32>| -> Result<()> {
            let mut proc = cmd_hidden(&ffmpeg)
                .args(
                    format!(
                        "-y -f f32le -ar {} -ac 2 -i pipe:0 -c:a pcm_f32le -f wav", sample_rate
                    )
                    .split_whitespace(),
                )
                .arg(output)
                .args(["-loglevel", "warning"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .with_context(|| tl!("run-ffmpeg-failed"))?;
            let input = proc.stdin.as_mut().unwrap();
            let slice = samples.as_slice().unwrap();
            let byte_slice = unsafe {
                std::slice::from_raw_parts(
                    slice.as_ptr() as *const u8,
                    std::mem::size_of_val(slice),
                )
            };
            input.write_all(byte_slice)?;
            proc.wait()?;
            Ok(())
        };

        output_audio(output_music_temp.path(), music_sample_rate, output_music)?;
        output_audio(output_sfx_temp.path(), sample_rate, output_sfx)?;
        output_audio(output_ending_temp.path(), ending_music_sample_rate, output_ending_music)?;

        eprintln!("Output Audio Time: {:.2?}", output_audio_time.elapsed());
    }

    if ipc {
        send(IPCEvent::Loading);
    }

    let preparing_render_time = Instant::now();
    let (vw, vh) = config.resolution;
    let mst = Rc::new(MSRenderTarget::new((vw, vh), config.sample_count));
    let my_time: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.));
    let tm = TimeManager::manual(Box::new({
        let my_time = Rc::clone(&my_time);
        move || *(*my_time).borrow()
    }));
    static MSAA: AtomicBool = AtomicBool::new(false);
    let player = build_player(&config).await?;
    let mut main = if config.render_loading {
        Main::new(
            Box::new(
                LoadingScene::new(
                    Some((chart, format)),
                    GameMode::Normal,
                    info,
                    &prpr_config,
                    fs,
                    Some(player),
                    None,
                    None,
                ).await?
            ),
            tm,
            {
                let mut cnt = 0;
                let mst = Rc::clone(&mst);
                move || {
                    cnt += 1;
                    if cnt == 1 || cnt == 3 {
                        MSAA.store(true, Ordering::SeqCst);
                        Some(mst.input())
                    } else {
                        MSAA.store(false, Ordering::SeqCst);
                        Some(mst.output())
                    }
                }
            },
        ).await?
    } else {
        let mut fs: Box<dyn FileSystem> = fs;
        let background = match LoadingScene::load_background(&mut fs, &prpr_config, &info.illustration).await {
            Ok((ill, bg)) => Some((ill, bg)),
            Err(err) => {
                warn!("failed to load background: {err:?}");
                None
            }
        };
        let (illustration, background): (SafeTexture, SafeTexture) = background
            .map(|(ill, back)| (ill.into(), back.into()))
            .unwrap_or_else(|| (BLACK_TEXTURE.clone(), BLACK_TEXTURE.clone()));
        Main::new(
            Box::new(
                GameScene::new(
                    Some((chart, format)),
                    GameMode::Normal,
                    info,
                    prpr_config,
                    fs,
                    Some(player),
                    background,
                    illustration,
                    None,
                    None,
                ).await?
            ),
            tm,
            {
                let mut cnt = 0;
                let mst = Rc::clone(&mst);
                move || {
                    cnt += 1;
                    if cnt == 1 || cnt == 3 {
                        MSAA.store(true, Ordering::SeqCst);
                        Some(mst.input())
                    } else {
                        MSAA.store(false, Ordering::SeqCst);
                        Some(mst.output())
                    }
                }
            },
        ).await?
    };
    main.top_level = false;
    main.viewport = Some((0, 0, vw as i32, vh as i32));

    let bitrate_control = if config.dynamic_bitrate_control {
        if ffmpeg_encoder == encoder_list[0] && !config.mpeg4 {
            "-cq"
        } else if ffmpeg_encoder == encoder_list[1]
            || config.mpeg4
            || ffmpeg_encoder == encoder_list[3]
        {
            "-q"
        } else if ffmpeg_encoder == encoder_list[2] {
            "-qp_p"
        } else if ffmpeg_encoder == config.custom_encoder.unwrap_or_default() {
            "-q"
        } else {
            "-crf"
        }
    } else {
        "-b:v"
    };

    let mut args = "-probesize 50M -y -f rawvideo -c:v rawvideo -color_range full".to_owned();
    if ffmpeg_encoder == encoder_list[0] {
        args += " -hwaccel_output_format cuda";
    }
    write!(
        &mut args,
        " -s {vw}x{vh} -r {fps} -pix_fmt yuv420p -thread_queue_size 1024 -i pipe:0"
    )?;

    let mut ffmpeg_audio_filter_music = if config.loudness_equalization { format!(
        "[2:a]loudnorm=I=-16:LRA=24:TP=-1,aresample={}:resampler=swr", sample_rate,
    )} else { format!(
        "[2:a]aresample={}:resampler=swr", sample_rate,
    )};

    let ffmpeg_audio_filter_music_volume = format!(",volume={}", volume_music);
    ffmpeg_audio_filter_music += &ffmpeg_audio_filter_music_volume;

    let ffmpeg_audio_filter_music_speed = if config.speed != 1.0 {
        format!(
            ",rubberband=tempo={}",
            config.speed
        )
    } else {
        String::new()
    };
    ffmpeg_audio_filter_music += &ffmpeg_audio_filter_music_speed;
    ffmpeg_audio_filter_music += "[a2];";

    let mut ffmpeg_audio_filter_sfx = format!(
            "[1:a]volume={}",
            volume_sfx
        );

    let ffmpeg_audio_filter_sfx_limit = if config.force_limit {
        format!(
            ",alimiter=limit={}:level=false:attack=0.1:release=1",
            config.limit_threshold
        )
    } else {
        String::new()
    };

    ffmpeg_audio_filter_sfx += &ffmpeg_audio_filter_sfx_limit;
    ffmpeg_audio_filter_sfx += "[a1];";

    let ffmpeg_audio_filter_ending =
        format!("[3:a]volume={},adelay={}[a3];", volume_music, output_ending_music_delay_string);

    let ffmpeg_audio_filter_mix = if config.hires {
        format!(
            "[a1][a2][a3]amix=inputs=3:duration=first:normalize=0[a]"
        )
    } else {
        format!(
            "[a1][a2][a3]amix=inputs=3:duration=first:normalize=0[aa];[aa]alimiter=limit=1.0:level=false:attack=0.1:release=1[a]"
        )
    };

    let ffmpeg_audio_filter = format!(
        "{}{}{}{}",
        ffmpeg_audio_filter_music,
        ffmpeg_audio_filter_sfx,
        ffmpeg_audio_filter_ending,
        ffmpeg_audio_filter_mix
    );

    let args2 = format!(
        "-c:a {} -c:v {} -movflags +faststart -pix_fmt yuv420p {} {} -filter_complex {} -map 0:v:0 -map [a] -f {}",
        if config.hires {
            "pcm_f32le"
        } else {
            "aac -b:a 320k"
        },
        ffmpeg_encoder,
        bitrate_control,
        config.bitrate,
        ffmpeg_audio_filter,
        if config.hires { "mov" } else { "mp4" }
    );

    eprintln!(
        "Preparing Render Time: {:.2?}",
        preparing_render_time.elapsed()
    );

    eprintln!("Command: {} {} {} {} {} {} {} {} {} {}",
        &ffmpeg,
        args,
        "-i", output_sfx_temp.path().display(),
        "-i", output_music_temp.path().display(),
        "-i", output_ending_temp.path().display(),
        args2,
        output_path.display()
    );

    let mut proc = cmd_hidden(&ffmpeg)
        .args(args.split_whitespace())
        .arg("-i").arg(output_sfx_temp.path())
        .arg("-i").arg(output_music_temp.path())
        .arg("-i").arg(output_ending_temp.path())
        .args(args2.split_whitespace())
        .arg(output_path)
        .args(["-loglevel", "warning"])
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| tl!("run-ffmpeg-failed"))?;
    let mut input = proc.stdin.take().unwrap();

    // let byte_size = (vw * vh * 4) as usize; // RGBA
    let yuvh = vh * 3 / 8; // (w * h * 3 / 2) / (w * 4) = h * 3 / 8
    let byte_size = (vw * vh * 3 / 2) as usize; // YUV420

    let yuv_target = render_target(vw, yuvh);
    let yuv_material = load_material(
        ShaderSource::Glsl {
            vertex: YUV_VERTEX_SHADER,
            fragment: YUV_FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("screenSize", UniformType::Int2),
                UniformDesc::new("targetSize", UniformType::Int2),
                UniformDesc::new("uFlipY", UniformType::Int1),
            ],
            textures: vec!["screenTexture".to_string()],
            ..Default::default()
        },
    )
    .with_context(|| "failed to load YUV shader")?;
    yuv_material.set_uniform("screenSize", [vw as i32, vh as i32]);
    yuv_material.set_uniform("targetSize", [vw as i32, yuvh as i32]);
    yuv_material.set_uniform("uFlipY", 1i32);

    const N: usize = 5; // Buffer Size
    let mut pbos: [GLuint; N] = [0; N];
    unsafe {
        use miniquad::gl::*;
        glGenBuffers(N as _, pbos.as_mut_ptr());
        for pbo in pbos {
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbo);
            glBufferData(
                GL_PIXEL_PACK_BUFFER,
                byte_size as _,
                std::ptr::null(),
                GL_STREAM_READ,
            );
        }
        glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
    }

    if ipc {
        send(IPCEvent::RenderFrame(video_frames));
    }
    let render_time = Instant::now();

    let fps = fps as f64;
    let frames_per_10 = (video_frames / 10).max(1);
    let frames = video_frames;
    let mut step_time = Instant::now();
    let mut last_print = Instant::now();
    let mut last_frame_progress = Instant::now();
    let mut pause_duration = Duration::ZERO;

    for frame in 0..frames {
        if !cmd && pause_requested.load(Ordering::SeqCst) {
            eprintln!("Render paused");
            if ipc { send(IPCEvent::Paused); }
            let pause_begin = Instant::now();
            while pause_requested.load(Ordering::SeqCst) {
                std::thread::park();
            }
            pause_duration += pause_begin.elapsed();
            step_time = Instant::now();
            eprintln!("Render resumed");
            if ipc { send(IPCEvent::Resumed); }
        }

        let now = (frame as f64) / fps;
        *my_time.borrow_mut() = now.max(0.);
        gl.quad_gl.render_pass(Some(mst.output().render_pass.raw_miniquad_id()));
        main.update()?;
        main.render(&mut painter)?;
        if *my_time.borrow() <= LoadingScene::TOTAL_TIME && config.render_loading {
            draw_rectangle(0., 0., 0., 0., Color::default());
        }
        gl.flush();

        if MSAA.load(Ordering::SeqCst) {
            mst.blit();
        }

        // GPU RGB -> YUV420
        yuv_material.set_texture("screenTexture", mst.output().texture);
        set_camera(&Camera2D {
            zoom: vec2(1., 1.),
            render_target: Some(yuv_target.clone()),
            ..Default::default()
        });
        gl_use_material(&yuv_material);
        draw_rectangle(-1., -1., 2., 2., WHITE);
        gl_use_default_material();
        gl.flush();

        if !cmd && frame % frames_per_10 == 0 {
            let progress = round_to_step((frame as f64 / video_frames as f64 * 100.).ceil(), 10.0);
            eprintln!("Render progress: {:.0}% {}/{} Time elapsed: {:.2}s",
                progress, frame, video_frames, std::mem::replace(&mut step_time, Instant::now()).elapsed().as_secs_f32());
        }

        if cmd && last_print.elapsed() >= Duration::from_secs(1) {
            let progress = (frame as f64 / video_frames as f64 * 100.).round();
            last_print = Instant::now();
            eprint!(
                "\rprogress={:.0}% frame={}/{}",
                progress, frame, video_frames
            );
        }

        unsafe {
            glBindFramebuffer(GL_READ_FRAMEBUFFER, internal_id(yuv_target.clone()));
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[frame as usize % N]);
            glReadPixels(
                0,
                0,
                vw as _,
                yuvh as _,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                std::ptr::null_mut(),
            );
            if frame >= N as u64 - 1 {
                glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[(frame + 1) as usize % N]);
                let src: *const u8 = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8 /* GL_READ_ONLY */);
                if !src.is_null() {
                    input.write_all(&std::slice::from_raw_parts(src, byte_size))?;
                }
                glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
            }
        }

        let completed = frame + 1;
        if ipc && (last_frame_progress.elapsed() >= Duration::from_millis(350) || completed == frames) {
            send(IPCEvent::Frame(completed));
            last_frame_progress = Instant::now();
        }
    }
    unsafe {
        let start = (frames as usize + 1) % N;
        for i in 0..N - 1 {
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[(start + i) % N]);
            let src: *const u8 = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8 /* GL_READ_ONLY */);
            if !src.is_null() {
                input.write_all(&std::slice::from_raw_parts(src, byte_size))?;
            }
            glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
        }
        glDeleteBuffers(N as _, pbos.as_ptr());
    }
    drop(input);
    if cmd {
        eprintln!(
            "\rprogress=100% frame={}/{}",
            video_frames, video_frames
        );
    }
    let actual_render_time = render_time.elapsed().saturating_sub(pause_duration);
    eprintln!("Render Time: {:.2?}", actual_render_time);
    eprintln!("Average FPS: {:.2}", frames as f64 / actual_render_time.as_secs_f64());
    proc.wait()?;
    eprintln!("Total Time: {:.2?}", loading_time.elapsed().saturating_sub(pause_duration));
    if ipc {
        send(IPCEvent::Done(render_start_time.elapsed().saturating_sub(pause_duration).as_secs_f64()));
    }
    Ok(())
}

const YUV_VERTEX_SHADER: &str = r#"
#version 130

in vec3 position;
in vec2 texcoord;

out vec2 fragTexCoord;

void main() {
    gl_Position = vec4(position, 1.0);
    fragTexCoord = texcoord;
}
"#;

const YUV_FRAGMENT_SHADER: &str = r#"
#version 130

// precision highp float;

in vec2 fragTexCoord;

uniform sampler2D screenTexture;
uniform ivec2 screenSize;
uniform ivec2 targetSize;
uniform bool uFlipY;

out vec4 outColor;

vec3 getPixel(int x, int y) {
    return texelFetch(screenTexture, ivec2(x, y), 0).xyz;
}

float getY(int x, int y) {
    vec3 pixel = getPixel(x, y);
    return dot(pixel, vec3(0.299, 0.587, 0.114));
}

float getU(int x, int y) {
    vec3 pixel = (
        getPixel(x, y)
        + getPixel(x, y + 1)
        + getPixel(x + 1, y)
        + getPixel(x + 1, y + 1)
    ) * 0.25;
    return dot(pixel, vec3(-0.168736, -0.331264, 0.5)) + 0.5;
}

float getV(int x, int y) {
    vec3 pixel = (
        getPixel(x, y)
        + getPixel(x, y + 1)
        + getPixel(x + 1, y)
        + getPixel(x + 1, y + 1)
    ) * 0.25;
    return dot(pixel, vec3(0.5, -0.418688, -0.081312)) + 0.5;
}

float getYI(int index) {
    return getY(index % screenSize.x, index / screenSize.x);
}

float getUI(int index) {
    return getU((index % (screenSize.x / 2)) * 2, index / (screenSize.x / 2) * 2);
}

float getVI(int index) {
    return getV((index % (screenSize.x / 2)) * 2, index / (screenSize.x / 2) * 2);
}

void main() {
    int w = screenSize.x; int h = screenSize.y;
    ivec2 curr_pos = ivec2(fragTexCoord * vec2(targetSize));
    if (!uFlipY) curr_pos.y = h - curr_pos.y - 1;
    int byte_index = (int(curr_pos.x) + int(curr_pos.y) * w) * 4;

    int y_bytes = w * h; int uv_bytes = y_bytes / 4;

    if (byte_index < y_bytes) {
        int pixel_index = byte_index;
        outColor = vec4(
            getYI(pixel_index), getYI(pixel_index + 1),
            getYI(pixel_index + 2), getYI(pixel_index + 3)
        );
    } else if (byte_index < y_bytes + uv_bytes) {
        int pixel_index = byte_index - y_bytes;
        outColor = vec4(
            getUI(pixel_index), getUI(pixel_index + 1),
            getUI(pixel_index + 2), getUI(pixel_index + 3)
        );
    } else if (byte_index < y_bytes + uv_bytes * 2) {
        int pixel_index = byte_index - y_bytes - uv_bytes;
        outColor = vec4(
            getVI(pixel_index), getVI(pixel_index + 1),
            getVI(pixel_index + 2), getVI(pixel_index + 3)
        );
    } else outColor = vec4(0);
}
"#;
