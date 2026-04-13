// Prevents additional console window on Windows in release, DO NOT REMOVE!!
phire::tl_file!("render");

use crate::{
    common::{get_output_dir, parse_args, read_config, test_output_dir},
    ipc::IPCEvent,
    task::generate_filename,
    ASSET_PATH
};
use anyhow::{bail, Context, Result};
use macroquad::{miniquad::gl::GLuint, prelude::*};
use ndarray::{s, Array1};
use phire::{
    config::{ChallengeModeColor, Config, Mods},
    core::{internal_id, HitSound, MSRenderTarget, Note, ResourcePack},
    ext::{NotNanExt, SafeTexture, BLACK_TEXTURE},
    fs::{self, FileSystem},
    info::ChartInfo,
    scene::{game::WAIT_TIME, BasicPlayer, EndingScene, GameMode, GameScene, LoadingScene},
    time::TimeManager,
    ui::{FontArc, TextPainter},
    Main,
};
use rustc_hash::FxHashMap;
use sasa::AudioClip;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    cmp::Ordering,
    io::{BufRead, BufWriter, Write},
    ops::DerefMut,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering as AtomicOrdering},
    time::Instant,
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
    pub compression_ratio: f32,
    pub force_limit: bool,
    pub limit_threshold: f32,
    pub loudness_equalization: bool,
    pub audio_mix_optimization: bool,
    pub watermark: String,
    pub roman: bool,
    pub chinese: bool,
    pub combo: String,
    pub difficulty: String,
    pub judge_offset: f32,
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
            play_start_time: self.play_start_time as f32,
            play_end_time: self.play_end_time.map(|v| v as f32),

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
            aggressive_chart: false,
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
            compression_ratio: 20.,
            force_limit: true,
            limit_threshold: 0.5,
            loudness_equalization: false,
            audio_mix_optimization: true,
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
    info!("Testing encoder: {}", encoder);
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

pub async fn main(cmd: bool) -> Result<()> {
    let loading_time = Instant::now();
    let asset_dir = ASSET_PATH.get().unwrap();
    set_pc_assets_folder(&asset_dir.display().to_string());
    let (mut fs, output_path, mut config, info) = if cmd {
        let (args_input, args_output, args_config, args_info) = parse_args(std::env::args().collect());

        let config: RenderConfig = if let Some(config) = &args_config {
            match serde_json::from_str(config) {
                Ok(config_json) => {
                    info!("Using config from json");
                    config_json
                }
                Err(error) => {
                    info!("Failed to parse json. Using config from toml file");
                    info!("{}", error);
                    toml::from_str(&std::fs::read_to_string(config)?)?
                }
            }
        } else {
            info!("Using config from config.toml");
            toml::from_str(&std::fs::read_to_string(std::env::current_exe()?.parent().unwrap().join("config.toml"))?)?
        };
        let path = args_input.unwrap();

        let mut fs = fs::fs_from_file(path.as_ref())?;

        let info = if let Some(info) = args_info {
            serde_json::from_str(&info)?
        } else {
            fs::load_info(fs.deref_mut()).await?
        };

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
        info!("output file: {:?}", output_path);

        (fs, output_path, config, info)
    } else {
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
    let Some(ffmpeg) = find_ffmpeg()? else {
        bail!("FFmpeg not found")
    };

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
    let speed_time_ratio = 1.0 / config.speed as f64;

    let before_time: f64 = if config.render_loading {
        LoadingScene::TOTAL_TIME as f64 + GameScene::BEFORE_DURATION as f64 * speed_time_ratio
    } else {
        0.0
    };
    let before_time_music: f64 = if config.render_loading {
        LoadingScene::TOTAL_TIME as f64 * speed + GameScene::BEFORE_DURATION as f64
    } else {
        0.0
    };

    let chart_length = before_time + config.play_end_time.unwrap_or(music_length).min(music_length) * speed_time_ratio - config.play_start_time * speed_time_ratio - offset as f64 + WAIT_TIME as f64 * speed_time_ratio;
    let chart_length_music = before_time_music + config.play_end_time.unwrap_or(music_length).min(music_length) - config.play_start_time - offset as f64 + WAIT_TIME as f64;
    let chart_length_sfx = config.play_end_time.unwrap_or(music_length).min(music_length) - config.play_start_time - offset as f64 + WAIT_TIME as f64;
    let video_length = chart_length + config.ending_length;
    let video_length_music = chart_length_music + config.ending_length; // chart_length needs to be divided by speed, but music needs to be rendered at the original speed, which is changed by ffmpeg
    let video_frames = (video_length * fps as f64 + N as f64 - 1.).ceil() as u64;

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

    info!("Loading Time: {:.2?}", loading_time.elapsed());
    info!("video length: {:.2}s frame: {}", video_length, video_frames);

    let render_start_time = Instant::now();

    if ipc {
        send(IPCEvent::Mixing);
    }

    let output_music_len = (video_length_music * music_sample_rate as f64).ceil() as usize * 2;
    let output_sfx_len = ((video_length + sfx_protect_time) * sample_rate_f64).ceil() as usize * 2;
    let output_ending_music_delay = chart_length + GameScene::WAIT_AFTER_TIME as f64 * speed_time_ratio + EndingScene::BPM_WAIT_TIME;
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
        let pos = (before_time - offset.min(0.) as f64) * speed;
        let position_wrtie = (pos * music_sample_rate as f64).ceil() as usize * 2;
        let position_read = ((offset.max(0.) as f64 + config.play_start_time) * music_sample_rate as f64).ceil() as usize * 2;
        let music_len = (chart_length_music * music_sample_rate as f64).ceil() as usize * 2;
        let len = (music.len() - position_read).min(output_music_len - position_wrtie).min(music_len - position_wrtie);
        let clip = music.slice(s![position_read..position_read + len]);
        let mut slice = output_music.slice_mut(s![position_wrtie..position_wrtie + len]);
        slice += &clip;
        info!("Process Music Time: {:.2?}", music_time.elapsed());
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
        let judge_offset = config.judge_offset as f64;
        let sfx_start_time = config.play_start_time as f32 - config.judge_offset;
        let sfx_end_time = sfx_start_time + chart_length_sfx as f32;
        let mut sfx_list: Vec<(f64, &Array1<f32>)> = Vec::new();

        if config.audio_mix_optimization {
            chart.lines.iter().flat_map(|line| &line.notes).filter(|note| !note.fake && note.time > sfx_start_time && note.time < sfx_end_time).for_each(|note| {
                if let Some(sfx) = get_hitsound(&note) {
                    sfx_list.push((before_time + note.time as f64 * speed_time_ratio + judge_offset - config.play_start_time * speed_time_ratio, sfx));
                }
            });
            let len = sfx_list.len();

            sfx_list.sort_by(|(a1, b1), (a2, b2)| {
                match a1.partial_cmp(a2).unwrap_or(Ordering::Equal) {
                    Ordering::Less  => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Equal => {
                        let p1 = b1.as_ptr() as usize;
                        let p2 = b2.as_ptr() as usize;
                        p1.cmp(&p2)
                    }
                }
            });

            let mut kept = Vec::with_capacity(sfx_list.len());
            let mut last_arr: Option<&Array1<f32>> = None;
            let mut last_t = 0.0;
            let mut count = 0;

            for &(pos, clip) in &sfx_list {
                let pos = round_to_step(pos, 0.005);
                let is_new_group = match last_arr {
                    None => true,
                    Some(prev) => {
                        !std::ptr::eq(prev, clip) || (pos - last_t).abs() > 0.005
                    }
                };

                if is_new_group {
                    last_arr = Some(clip);
                    last_t = pos;
                    count = 1;
                    kept.push((pos, clip));
                } else {
                    count += 1;
                    if count <= 3 {
                        kept.push((pos, clip));
                    }
                }
            }

            sfx_list = kept;
            let num = sfx_list.len();
            if ipc {
                send(IPCEvent::MixingSfx(num as u64));
            }
            for (pos, sfx) in sfx_list {
                place_sfx(pos, sfx);
                if ipc {
                    send(IPCEvent::Sfx);
                }
            }

            let elapsed = sfx_time.elapsed();
            info!("Process Hit Effects Time: {:.2?} Equivalent Speed: {:.2} notes/sec Speed: {:.2} notes/sec", elapsed, len as f32 / elapsed.as_secs_f32(), num as f32 / elapsed.as_secs_f32())
        } else {
            chart.lines.iter().flat_map(|line| &line.notes).filter(|note| !note.fake && note.time > sfx_start_time && note.time < sfx_end_time).for_each(|note| {
                if let Some(sfx) = get_hitsound(&note) {
                    sfx_list.push((before_time + note.time as f64 * speed_time_ratio + judge_offset - config.play_start_time * speed_time_ratio, sfx));
                }
            });
            let num = sfx_list.len();
            if ipc {
                send(IPCEvent::MixingSfx(num as u64));
            }
            for (pos, sfx) in sfx_list {
                place_sfx(pos, sfx);
                if ipc {
                    send(IPCEvent::Sfx);
                }
            }

            let elapsed = sfx_time.elapsed();
            info!("Process Hit Effects Time: {:.2?} Speed: {:.2} notes/sec", elapsed, num as f32 / elapsed.as_secs_f32())
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
        info!("Process Ending Music Time: {:.2?}", ending_time.elapsed());
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
            let mut writer = BufWriter::new(input);
            for sample in samples {
                writer.write_all(&sample.to_le_bytes())?;
            }
            drop(writer);
            proc.wait()?;
            Ok(())
        };

        output_audio(output_music_temp.path(), music_sample_rate, output_music)?;
        output_audio(output_sfx_temp.path(), sample_rate, output_sfx)?;
        output_audio(output_ending_temp.path(), ending_music_sample_rate, output_ending_music)?;

        info!("Output Audio Time: {:.2?}", output_audio_time.elapsed());
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
                        MSAA.store(true, AtomicOrdering::SeqCst);
                        Some(mst.input())
                    } else {
                        MSAA.store(false, AtomicOrdering::SeqCst);
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
                        MSAA.store(true, AtomicOrdering::SeqCst);
                        Some(mst.input())
                    } else {
                        MSAA.store(false, AtomicOrdering::SeqCst);
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

    let mut args = "-probesize 50M -y -f rawvideo -c:v rawvideo".to_owned();
    if ffmpeg_encoder == encoder_list[0] {
        args += " -hwaccel_output_format cuda";
    }
    write!(
        &mut args,
        " -s {vw}x{vh} -r {fps} -pix_fmt rgba -thread_queue_size 1024 -i pipe:0"
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
    } else if config.compression_ratio > 1. {
        format!(
            ",acompressor=threshold=0dB:ratio={}:attack=0.01:release=0.01",
            config.compression_ratio
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
        "-c:a {} -c:v {} -pix_fmt yuv420p {} {} -filter_complex {} -map 0:v:0 -map [a] -vf vflip -f {}",
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

    info!(
        "Preparing Render Time: {:.2?}",
        preparing_render_time.elapsed()
    );

    info!("Command: {} {} {} {} {} {} {} {} {} {}",
        &ffmpeg,
        args,
        "-i", output_sfx_temp.path().display(),
        "-i", output_music_temp.path().display(),
        "-i", ASSET_PATH.get().unwrap().join("ending.ogg").display(),
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

    let byte_size = vw as usize * vh as usize * 4;

    const N: usize = 5; // Buffer Size
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
        send(IPCEvent::RenderFrame(video_frames));
    }
    let render_time = Instant::now();

    let fps = fps as f64;
    let frames_per_10 = (video_frames / 10).max(1);
    let frames = video_frames;
    let mut step_time = Instant::now();
    for frame in 0..frames {
        let now = (frame as f64) / fps;
        *my_time.borrow_mut() = now.max(0.);
        gl.quad_gl.render_pass(Some(mst.output().render_pass));
        main.update()?;
        main.render(&mut painter)?;
        if *my_time.borrow() <= LoadingScene::TOTAL_TIME as f64 && config.render_loading {
            draw_rectangle(0., 0., 0., 0., Color::default());
        }
        gl.flush();

        if MSAA.load(AtomicOrdering::SeqCst) {
            mst.blit();
        }

        if frame % frames_per_10 == 0 {
            let progress = round_to_step((frame as f64 / video_frames as f64 * 100.).ceil(), 10.0);
            info!("Render progress: {:.0}% {}/{} Time elapsed: {:.2}s",
                progress, frame, video_frames, std::mem::replace(&mut step_time, Instant::now()).elapsed().as_secs_f32());
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
            if frame >= N as u64 {
                glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[(frame + 1) as usize % N]);
                let src: *const u8 = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8 /* GL_READ_ONLY */);
                if !src.is_null() {
                    input.write_all(&std::slice::from_raw_parts(src, byte_size))?;
                }
                glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
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
    if ipc {
        send(IPCEvent::Done(render_start_time.elapsed().as_secs_f64()));
    }
    Ok(())
}
