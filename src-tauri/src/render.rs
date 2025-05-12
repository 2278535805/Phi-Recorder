// Prevents additional console window on Windows in release, DO NOT REMOVE!!
prpr::tl_file!("render");

use crate::{
    common::{output_dir, read_config, test_output_dir},
    ASSET_PATH,
};
use anyhow::{bail, Context, Result};
use chrono::Local;
use macroquad::{miniquad::gl::GLuint, prelude::*};
use prpr::{
    config::{ChallengeModeColor, Config, Mods},
    core::{init_assets, internal_id, HitSound, MSRenderTarget, Note, ResourcePack},
    fs,
    info::ChartInfo,
    scene::{BasicPlayer, EndingScene, GameMode, GameScene, LoadingScene},
    time::TimeManager,
    ui::{FontArc, TextPainter},
    Main,
};
use sasa::AudioClip;
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    cell::RefCell,
    io::{BufRead, BufWriter, Write},
    ops::DerefMut,
    path::PathBuf,
    process::{Command, Stdio},
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
use std::{ffi::OsStr, fmt::Write as _};
use tempfile::NamedTempFile;
use toml::Value;

fn deserialize_f32_or_default<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Float(f) => Ok(f as f32),
        Value::Integer(i) => Ok(i as f32),
        Value::Boolean(b) => Ok(if b { 0.2 } else { 0.0 }),
        _ => Ok(0.0),
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct RenderConfig {
    pub resolution: (u32, u32),
    pub ffmpeg_preset: String,
    pub ending_length: f64,
    pub disable_loading: bool,
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

    pub aggressive: bool,
    pub challenge_color: ChallengeModeColor,
    pub challenge_rank: u32,
    pub disable_effect: bool,
    pub double_hint: bool,
    pub fxaa: bool,
    pub note_scale: f32,
    //pub offset: f32,
    pub particle: bool,
    pub player_avatar: Option<String>,
    pub player_name: String,
    pub player_rks: f32,
    pub sample_count: u32,
    pub res_pack_path: Option<String>,
    pub speed: f32,
    pub volume_music: f32,
    pub volume_sfx: f32,
    pub compression_ratio: f32,
    pub force_limit: bool,
    pub limit_threshold: f32,
    pub loudness_equalization: bool,
    pub watermark: String,
    pub roman: bool,
    pub chinese: bool,
    pub combo: String,
    pub difficulty: String,
    pub judge_offset: f32,
    pub simple_file_name: bool,

    pub render_line: bool,
    pub render_line_extra: bool,
    pub render_note: bool,
    pub render_ui_pause: bool,
    pub render_ui_name: bool,
    pub render_ui_level: bool,
    pub render_ui_score: bool,
    pub render_ui_combo: bool,
    pub render_ui_bar: bool,
    pub render_bg: bool,
    pub render_bg_dim: bool,
    pub bg_blurriness: f32,

    pub max_particles: usize,
    pub render_start_time: f64,
    pub render_end_time: Option<f64>,

    pub fade: f32,
    pub alpha_tint: bool,
}

impl RenderConfig {
    pub fn to_config(&self) -> Config {
        Config {
            aggressive: self.aggressive,
            challenge_color: self.challenge_color.clone(),
            challenge_rank: self.challenge_rank,
            disable_effect: self.disable_effect,
            disable_loading: self.disable_loading,
            hires: self.hires,
            double_hint: self.double_hint,
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
            disable_audio: false,
            judge_offset: self.judge_offset,

            render_line: self.render_line,
            render_line_extra: self.render_line_extra,
            render_note: self.render_note,
            render_ui_pause: self.render_ui_pause,
            render_ui_name: self.render_ui_name,
            render_ui_level: self.render_ui_level,
            render_ui_score: self.render_ui_score,
            render_ui_combo: self.render_ui_combo,
            render_ui_bar: self.render_ui_bar,
            render_bg: self.render_bg,
            render_bg_dim: self.render_bg_dim,
            bg_blurriness: self.bg_blurriness,

            max_particles: self.max_particles,
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
            ffmpeg_preset: "medium".to_string(),
            ending_length: 5.0,
            disable_loading: false,
            hires: false,
            fps: 60,
            hardware_accel: true,
            hevc: false,
            mpeg4: false,
            custom_encoder: None,
            dynamic_bitrate_control: true,
            bitrate: "28".to_string(),
            aggressive: false,
            challenge_color: ChallengeModeColor::Rainbow,
            challenge_rank: 45,
            disable_effect: false,
            double_hint: true,
            fxaa: false,
            note_scale: 1.0,
            particle: true,
            player_name: "HLMC".to_string(),
            player_rks: 16.0,
            sample_count: 8,
            res_pack_path: None,
            speed: 1.0,
            volume_music: 1.0,
            volume_sfx: 0.7,
            compression_ratio: 20.,
            force_limit: false,
            limit_threshold: 1.0,
            loudness_equalization: false,
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
            simple_file_name: false,

            render_line: true,
            render_line_extra: true,
            render_note: true,
            render_ui_pause: true,
            render_ui_name: true,
            render_ui_level: true,
            render_ui_score: true,
            render_ui_combo: true,
            render_ui_bar: true,
            render_bg: true,
            render_bg_dim: true,
            bg_blurriness: 80.,

            max_particles: 100000,
            render_start_time: 0.0,
            render_end_time: None,

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

#[derive(Serialize, Deserialize)]
pub enum IPCEvent {
    Loading,
    StartMixing,
    StartRender(u64),
    Frame,
    Done(f64),
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

pub fn find_ffmpeg() -> Result<Option<String>> {
    fn test(path: impl AsRef<OsStr>) -> bool {
        matches!(cmd_hidden(path).arg("-version").output(), Ok(_))
    }
    if test("ffmpeg") {
        return Ok(Some("ffmpeg".to_owned()));
    }
    let exe_dir = std::env::current_exe()?.parent().unwrap().to_owned();
    let ffmpeg = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    let ffmpeg = exe_dir.join(ffmpeg);
    Ok(if test(&ffmpeg) {
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

    let test_encoder = |encoder: &str| -> bool {
        info!("Testing encoder: {}", encoder);
        let output = Command::new(&ffmpeg)
            .args([
                "-f",
                "lavfi",
                "-i",
                "testsrc=size=1920x1080:rate=5:duration=1",
                "-pix_fmt",
                "yuv420p",
                "-c:v",
                encoder,
                "-f",
                "null",
                "-",
            ])
            .arg("-loglevel")
            .arg("error")
            .arg("-hide_banner")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .with_context(|| tl!("run-ffmpeg-failed"))
            .expect("failed test encoder");

        output.status.success()
    };

    for encoder in encoder_list {
        if test_encoder(encoder) {
            return Some(encoder.to_string());
        } else {
            warn!("Encoder {} not supported", encoder);
        }
    }

    None
}

pub fn test_encoder(ffmpeg: String, encoder: String) -> bool {
    info!("Testing encoder: {}", encoder);
    let output = Command::new(&ffmpeg)
        .args([
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=1920x1080:rate=5:duration=1",
            "-pix_fmt",
            "yuv420p",
            "-c:v",
            encoder.as_str(),
            "-f",
            "null",
            "-",
        ])
        .arg("-loglevel")
        .arg("info")
        // .arg("-hide_banner")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .with_context(|| tl!("run-ffmpeg-failed"))
        .expect("failed test encoder");
    output.status.success()
}

pub async fn main(cmd: bool) -> Result<()> {
    let loading_time = Instant::now();

    let (mut fs, output_path, mut config, info) = if cmd {
        init_assets();

        let args: Vec<String> = std::env::args().collect();
        let mut args_input = None;
        let mut args_output = None;
        let mut args_config = None;

        let mut args_now = 1;
        while args_now < args.len() {
            match args[args_now].as_str() {
                "--output" => {
                    args_output = args.get(args_now + 1).cloned();
                    args_now += 2;
                }
                "--config" => {
                    args_config = args.get(args_now + 1).cloned();
                    args_now += 2;
                }
                arg => {
                    if !arg.starts_with("--") && args_input.is_none() {
                        args_input = Some(arg.to_string());
                    }
                    args_now += 1;
                }
            }
        }

        let config: RenderConfig = if let Some(config) = &args_config {
            match serde_json::from_str(config) {
                Ok(config_json) => {
                    println!("Using config from json");
                    config_json
                }
                Err(error) => {
                    println!("Failed to parse json: {}", error);
                    println!("Using config from toml file");
                    toml::from_str(&std::fs::read_to_string(config)?)?
                }
            }
        } else {
            println!("Using config from config.toml");
            toml::from_str(&std::fs::read_to_string("config.toml")?)?
        };
        let path = args_input.unwrap();

        let mut fs = fs::fs_from_file(path.as_ref())?;
        let info = fs::load_info(fs.deref_mut()).await?;
        let level: String = info
            .level
            .split_whitespace()
            .next()
            .unwrap_or("UK")
            .to_string();
        let safe_name: String = info
            .name
            .chars()
            .filter(|&it| it == '-' || it == '_' || it.is_alphanumeric())
            .collect();
        let format = if config.hires { "mov" } else { "mp4" };
        let file_name = if config.simple_file_name {
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

        let output_path = if let Some(output_string) = args_output {
            let output_dir = PathBuf::from(output_string);
            test_output_dir(output_dir.clone())?;
            output_dir.join(file_name)
        } else {
            if let Some(set_output_dir) = read_config()?.output_dir {
                set_output_dir.join(file_name)
            } else {
                output_dir()?.join(file_name)
            }
        };
        info!("output file: {:?}", output_path);

        (fs, output_path, config, info)
    } else {
        set_pc_assets_folder(&std::env::args().nth(2).unwrap());

        let mut stdin = std::io::stdin().lock();
        let stdin = &mut stdin;

        let mut line = String::new();
        stdin.read_line(&mut line)?;
        let params: RenderParams = serde_json::from_str(line.trim())?;
        let path = params.path;

        line.clear();
        stdin.read_line(&mut line)?;
        let output_path: PathBuf = serde_json::from_str(line.trim())?;

        let fs = fs::fs_from_file(&path)?;

        let config = params.config;
        let info = params.info;

        (fs, output_path, config, info)
    };

    use crate::ipc::client::*;
    let ipc = if cmd { false } else { true };
    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;
    let mut painter = TextPainter::new(font);
    let volume_music = std::mem::take(&mut config.volume_music);
    let volume_sfx = std::mem::take(&mut config.volume_sfx);
    let mut prpr_config = config.to_config();
    prpr_config.mods = Mods::AUTOPLAY;
    prpr_config.disable_audio = true;
    let Some(ffmpeg) = find_ffmpeg()? else {
        bail!("FFmpeg not found")
    };

    let (mut chart, ..) = GameScene::load_chart(fs.deref_mut(), &info)
        .await
        .with_context(|| tl!("load-chart-failed"))?;
    let res_pack = ResourcePack::from_path(config.res_pack_path.as_ref())
        .await
        .context("Failed to load resource pack")?;
    let music: Result<_> = async { AudioClip::new(fs.load_file(&info.music).await?) }.await;
    let music = music.with_context(|| tl!("load-music-failed"))?;
    let music_length = music.length() as f64;
    let music_sample_rate = music.sample_rate();
    let ending_music = res_pack.ending;
    let sfx_click = res_pack.sfx_click;
    let sfx_drag = res_pack.sfx_drag;
    let sfx_flick = res_pack.sfx_flick;

    let mut gl = unsafe { get_internal_gl() };

    let before_time: f64 = if config.disable_loading {
        GameScene::BEFORE_DURATION as f64
    } else {
        LoadingScene::TOTAL_TIME as f64 + GameScene::BEFORE_DURATION as f64
    };
    let cut_time: f64 = if config.disable_loading {
        GameScene::BEFORE_DURATION as f64 + config.render_start_time
    } else {
        0.0
    };
    let fade_out_time: f64 = -0.5;

    let fps = config.fps;
    let offset = chart.offset + info.offset;
    let chart_length = before_time + config.render_end_time.unwrap_or(music_length).min(music_length) - offset as f64 + 1.;
    let video_length = chart_length + fade_out_time + config.ending_length;
    let frames = (video_length * fps as f64 + N as f64 - 1.).ceil() as u64;

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

    info!("Encoder: {}", ffmpeg_encoder);

    info!("Loading Time:{:.2?}", loading_time.elapsed());
    info!("video length: {:.2}s", video_length);

    let render_start_time = Instant::now();

    if ipc {
        send(IPCEvent::StartMixing);
    }
    let sample_rate = 48000;
    let sample_rate_f64 = sample_rate as f64;
    assert_eq!(
        sample_rate,
        ending_music.sample_rate(),
        "Sample rate mismatch: expected {}, got {}",
        sample_rate,
        ending_music.sample_rate()
    );
    assert_eq!(
        sample_rate,
        sfx_click.sample_rate(),
        "Sample rate mismatch: expected {}, got {}",
        sample_rate,
        sfx_click.sample_rate()
    );
    assert_eq!(
        sample_rate,
        sfx_drag.sample_rate(),
        "Sample rate mismatch: expected {}, got {}",
        sample_rate,
        sfx_drag.sample_rate()
    );
    assert_eq!(
        sample_rate,
        sfx_flick.sample_rate(),
        "Sample rate mismatch: expected {}, got {}",
        sample_rate,
        sfx_flick.sample_rate()
    );

    let mut output_music =
        vec![0.0_f32; (video_length * music_sample_rate as f64).ceil() as usize * 2];
    let mut output_fx = vec![0.0_f32; (video_length * sample_rate_f64).ceil() as usize * 2];

    // let stereo_sfx = false; // TODO stereo sound effects
    let mut place_fx = |pos: f64, clip: &AudioClip| {
        let position = (pos * sample_rate_f64).round() as usize * 2;
        if position >= output_fx.len() {
            return 0;
        }
        let slice = &mut output_fx[position..];
        let len = (slice.len() / 2).min(clip.frame_count());

        let frames = clip.frames();
        for i in 0..len {
            slice[i * 2] += frames[i].0;
            slice[i * 2 + 1] += frames[i].1;
        }

        return len;
    };

    if volume_music != 0.0 {
        let music_time = Instant::now();
        let pos = before_time - offset.min(0.) as f64;
        let len = ((music_length + config.ending_length) * music_sample_rate as f64) as usize;
        let start_index = (pos * music_sample_rate as f64).round() as usize * 2;
        let ratio = 1.0 / music_sample_rate as f64;
        let slice = &mut output_music[start_index..];
        for i in 0..len.min(slice.len() / 2) {
            let position = i as f64 * ratio + offset.max(0.) as f64;
            let frame = music.sample_f64(position).unwrap_or_default();
            slice[i * 2] += frame.0;
            slice[i * 2 + 1] += frame.1;
        }
        info!("Process Music Time:{:.2?}", music_time.elapsed())
    }

    type AudioMap = std::collections::HashMap<String, AudioClip>;
    let mut extra_sfxs: AudioMap = AudioMap::new();

    chart.hitsounds.drain().for_each(|(name, clip)| {
        extra_sfxs.insert(name, clip);
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
        let judge_offset = config.judge_offset as f64;
        for line in &chart.lines {
            for note in &line.notes {
                if !note.fake {
                    if let Some(sfx) = get_hitsound(note) {
                        if note.time as f64 > chart_length {
                            continue;
                        }
                        place_fx(before_time + note.time as f64 + judge_offset, sfx);
                    }
                }
            }
        }
        info!("Process Hit Effects Time:{:.2?}", sfx_time.elapsed())
    }

    let output_music_temp = NamedTempFile::new()?;
    let output_fx_temp = NamedTempFile::new()?;

    {
        let output_audio_time = Instant::now();

        let mut proc = cmd_hidden(&ffmpeg)
            .args(
                format!(
                    "-y -f f32le -ar {} -ac 2 -i pipe:0 -c:a pcm_f32le -f wav",
                    music_sample_rate
                )
                .split_whitespace(),
            )
            .arg(output_music_temp.path())
            .arg("-loglevel")
            .arg("warning")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| tl!("run-ffmpeg-failed"))?;
        let input = proc.stdin.as_mut().unwrap();
        let mut writer = BufWriter::new(input);
        for sample in output_music.into_iter() {
            writer.write_all(&sample.to_le_bytes())?;
        }
        drop(writer);
        proc.wait()?;

        let mut proc = cmd_hidden(&ffmpeg)
            .args(
                format!(
                    "-y -f f32le -ar {} -ac 2 -i pipe:0 -c:a pcm_f32le -f wav",
                    sample_rate
                )
                .split_whitespace(),
            )
            .arg(output_fx_temp.path())
            .arg("-loglevel")
            .arg("warning")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| tl!("run-ffmpeg-failed"))?;
        let input = proc.stdin.as_mut().unwrap();
        let mut writer = BufWriter::new(input);
        for sample in output_fx.into_iter() {
            writer.write_all(&sample.to_le_bytes())?;
        }
        drop(writer);
        proc.wait()?;

        info!("Output Audio Time:{:.2?}", output_audio_time.elapsed());
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
    let mut main = Main::new(
        Box::new(
            LoadingScene::new(
                GameMode::Normal,
                info,
                &prpr_config,
                fs,
                Some(player),
                None,
                None,
            )
            .await?,
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
    )
    .await?;
    main.top_level = false;
    main.viewport = Some((0, 0, vw as _, vh as _));

    let ffmpeg_preset = "-preset";
    let ffmpeg_preset_name_list: Vec<String> = config
        .ffmpeg_preset
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let ffmpeg_preset_name = if ffmpeg_encoder == encoder_list[0] {
        if let Some(i) = ffmpeg_preset_name_list.get(1) {
            i.as_str()
        } else if let Some(i) = ffmpeg_preset_name_list.get(0) {
            i.as_str()
        } else {
            "p4"
        }
    } else if ffmpeg_encoder == encoder_list[1] {
        if let Some(i) = ffmpeg_preset_name_list.get(2) {
            i.as_str()
        } else if let Some(i) = ffmpeg_preset_name_list.get(0) {
            i.as_str()
        } else {
            "medium"
        }
    } else if ffmpeg_encoder == encoder_list[2] {
        if let Some(i) = ffmpeg_preset_name_list.get(3) {
            i.as_str()
        } else if let Some(i) = ffmpeg_preset_name_list.get(0) {
            i.as_str()
        } else {
            "balanced"
        }
    } else {
        if let Some(i) = ffmpeg_preset_name_list.get(0) {
            i.as_str()
        } else {
            "medium"
        }
    };

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

    let mut args = "-probesize 50M -y -f rawvideo -c:v rawvideo".to_owned();
    if ffmpeg_encoder == encoder_list[0] {
        args += " -hwaccel_output_format cuda";
    }
    write!(
        &mut args,
        " -s {vw}x{vh} -r {fps} -pix_fmt rgba -thread_queue_size 1024 -i pipe:0"
    )?;

    let delay_ending =
        (chart_length + GameScene::WAIT_AFTER_TIME as f64 + EndingScene::BPM_WAIT_TIME) * 1000.;
    let delay_ending = format!("{}|{}", delay_ending, delay_ending);

    let mut ffmpeg_audio_filter_music = format!(
        "[1:a]aresample=48000:resampler=soxr:precision=28,volume={}",
        volume_music
    );
    if config.loudness_equalization {
        ffmpeg_audio_filter_music += ",loudnorm=I=-14:LRA=12:TP=-1"
    }
    ffmpeg_audio_filter_music += "[a1];";

    let ffmpeg_audio_filter_fx = if config.force_limit {
        format!(
            "[2:a]volume={},alimiter=limit={}:level=false:attack=0.1:release=1[a2];",
            config.limit_threshold, volume_sfx
        )
    } else if config.compression_ratio > 1. {
        format!(
            "[2:a]volume={},acompressor=threshold=0dB:ratio={}:attack=0.01:release=0.01[a2];",
            config.compression_ratio, volume_sfx
        )
    } else {
        format!("[2:a]volume={}", volume_sfx)
    };

    let ffmpeg_audio_filter_ending =
        format!("[3:a]volume={},adelay={}[a3];", delay_ending, volume_music);

    let ffmpeg_audio_effect_mix = if config.hires {
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
        ffmpeg_audio_filter_fx,
        ffmpeg_audio_filter_ending,
        ffmpeg_audio_effect_mix
    );

    let args2 = format!(
        "-c:a {} -c:v {} -pix_fmt yuv420p {} {} {} {} -filter_complex {} -map 0:v:0 -map [a] {} -vf vflip -f {}",
        if config.hires {
            "pcm_f32le"
        } else {
            "aac -b:a 320k"
        },
        ffmpeg_encoder,
        bitrate_control,
        config.bitrate,
        ffmpeg_preset,
        ffmpeg_preset_name,
        ffmpeg_audio_filter,
        format!("-ss {}", cut_time),
        if config.hires { "mov" } else { "mp4" }
    );

    info!(
        "Preparing Render Time:{:.2?}",
        preparing_render_time.elapsed()
    );

    //info!("Command: {} {} {} {} {}", "ffmpeg", args,"-", args2, output_path.display());
    let mut proc = cmd_hidden(&ffmpeg)
        .args(args.split_whitespace())
        .arg("-i")
        .arg(output_music_temp.path())
        .arg("-i")
        .arg(output_fx_temp.path())
        .arg("-i")
        .arg(ASSET_PATH.get().unwrap().join("ending.ogg"))
        .args(args2.split_whitespace())
        .arg(output_path)
        .arg("-loglevel")
        .arg("warning")
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| tl!("run-ffmpeg-failed"))?;
    let mut input = proc.stdin.take().unwrap();

    let byte_size = vw as usize * vh as usize * 4;

    const N: usize = 6;
    let mut pbos: [GLuint; N] = [0; N];
    unsafe {
        use miniquad::gl::*;
        glGenBuffers(N as _, pbos.as_mut_ptr());
        for pbo in pbos {
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbo);
            glBufferData(
                GL_PIXEL_PACK_BUFFER,
                (vw as u64 * vh as u64 * 4) as _,
                std::ptr::null(),
                GL_STREAM_READ,
            );
        }
        glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
    }

    if ipc {
        send(IPCEvent::StartRender(frames));
    }
    let render_time = Instant::now();

    let fps = fps as f64;
    let frames10 = frames / 10;
    let mut step_time = Instant::now();
    for frame in 0..frames {
        let now = (frame as f64) / fps;
        *my_time.borrow_mut() = now.max(0.);
        gl.quad_gl.render_pass(Some(mst.output().render_pass));
        main.update()?;
        main.render(&mut painter)?;
        if *my_time.borrow() <= LoadingScene::TOTAL_TIME as f64 && !config.disable_loading {
            draw_rectangle(0., 0., 0., 0., Color::default());
        }
        gl.flush();

        if MSAA.load(Ordering::SeqCst) {
            mst.blit();
        }

        if frame % frames10 == 0 {
            let progress = (frame as f32 / frames as f32 * 100.).ceil() as u8;
            info!("Render progress: {}% Time elapsed: {:.2}s", progress, 
                std::mem::replace(&mut step_time, Instant::now()).elapsed().as_secs_f32());
        }

        unsafe {
            use miniquad::gl::*;
            glBindFramebuffer(GL_READ_FRAMEBUFFER, internal_id(mst.output()));
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[frame as usize % N]);
            glReadPixels(
                0,
                0,
                vw as _,
                vh as _,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                std::ptr::null_mut(),
            );

            if frame >= N as u64 - 1 {
                glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[(frame + 1 ) as usize % N]);
                let src = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8);
                if !src.is_null() {
                    input.write_all(&std::slice::from_raw_parts(src as *const u8, byte_size))?;
                    glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
                }
            }
        }

        if ipc {
            send(IPCEvent::Frame);
        }
    }
    drop(input);
    info!("Render Time: {:.2?}", render_time.elapsed());
    info!(
        "Average FPS: {:.2}",
        frames as f64 / render_time.elapsed().as_secs_f64()
    );
    proc.wait()?;
    info!("Task done in {:.2?}", render_start_time.elapsed());
    if ipc {
        send(IPCEvent::Done(render_start_time.elapsed().as_secs_f64()));
    }
    Ok(())
}
