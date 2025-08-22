// Prevents additional console window on Windows in release, DO NOT REMOVE!!
phire::tl_file!("render");

use crate::{
    common::{output_dir, parse_args, read_config, test_output_dir}, ipc::IPCEvent, task::generate_filename, ASSET_PATH
};
use anyhow::{bail, Context, Result};
use macroquad::{miniquad::gl::GLuint, prelude::*};
use ndarray::{s, Array1};
use phire::{
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
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashMap,
    cmp::Ordering,
    io::{BufRead, BufWriter, Write},
    ops::DerefMut,
    path::PathBuf,
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
    pub audio_mix_optimization: bool,
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
    pub render_double_hint: bool,
    pub render_ui_pause: bool,
    pub render_ui_name: bool,
    pub render_ui_level: bool,
    pub render_ui_score: bool,
    pub render_ui_combo: bool,
    pub render_ui_bar: bool,
    pub render_bg: bool,
    pub render_bg_dim: bool,
    pub render_extra: bool,
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
            disable_loading: self.disable_loading,
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
            render_double_hint: self.render_double_hint,
            render_ui_pause: self.render_ui_pause,
            render_ui_name: self.render_ui_name,
            render_ui_level: self.render_ui_level,
            render_ui_score: self.render_ui_score,
            render_ui_combo: self.render_ui_combo,
            render_ui_bar: self.render_ui_bar,
            render_bg: self.render_bg,
            render_bg_dim: self.render_bg_dim,
            render_extra: self.render_extra,
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
            ending_length: 0.0,
            disable_loading: true,
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
            simple_file_name: false,

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
            render_extra: true,
            bg_blurriness: 80.,

            max_particles: 5000,
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
        .arg("warning")
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

        let (args_input, args_output, args_config) = parse_args(std::env::args().collect());

        let config: RenderConfig = if let Some(config) = &args_config {
            match serde_json::from_str(config) {
                Ok(config_json) => {
                    info!("Using config from json");
                    config_json
                }
                Err(error) => {
                    info!("Failed to parse json: {}. Using config from toml file", error);
                    toml::from_str(&std::fs::read_to_string(config)?)?
                }
            }
        } else {
            info!("Using config from config.toml");
            toml::from_str(&std::fs::read_to_string("config.toml")?)?
        };
        let path = args_input.unwrap();

        let mut fs = fs::fs_from_file(path.as_ref())?;
        let info = fs::load_info(fs.deref_mut()).await?;

        let file_name = generate_filename(&info, &config);

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
    let sfx_click = res_pack.sfx_click;
    let sfx_drag = res_pack.sfx_drag;
    let sfx_flick = res_pack.sfx_flick;

    let sample_rate = 48000;
    let sample_rate_f64 = sample_rate as f64;

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
    let sfx_click = Array1::from_vec(sfx_click.to_vec());
    let sfx_drag = Array1::from_vec(sfx_drag.to_vec());
    let sfx_flick = Array1::from_vec(sfx_flick.to_vec());

    let mut gl = unsafe { get_internal_gl() };

    let before_time: f64 = if config.disable_loading {
        0.0
    } else {
        LoadingScene::TOTAL_TIME as f64 + GameScene::BEFORE_DURATION as f64
    };
    let video_cut_time: f64 = if config.disable_loading {
        config.render_start_time
    } else {
        0.0
    };

    let fps = config.fps;
    let offset = chart.offset + info.offset;
    let chart_length = before_time + config.render_end_time.unwrap_or(music_length).min(music_length) - offset as f64;
    let video_length = chart_length + config.ending_length - video_cut_time;
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

    info!("Loading Time: {:.2?}", loading_time.elapsed());
    info!("video length: {:.2}s", video_length);

    let render_start_time = Instant::now();

    if ipc {
        send(IPCEvent::StartMixing);
    }

    let output_music_len = ((video_length + video_cut_time) * music_sample_rate as f64).ceil() as usize * 2;
    let output_fx_len = ((video_length + video_cut_time + 1.0) * sample_rate_f64).ceil() as usize * 2;

    let mut output_music = Array1::from_vec(vec![0.0_f32; output_music_len]);
    let mut output_fx = Array1::from_vec(vec![0.0_f32; output_fx_len]);

    let mut place_fx = |pos: f64, clip: &Array1<f32>| {
        let position = (pos * sample_rate_f64).ceil() as usize * 2;
        let len = clip.len();
        let mut slice = output_fx.slice_mut(s![position..position + len]);
        slice += clip;
    };

    if volume_music != 0.0 {
        let music_time = Instant::now();
        let pos = before_time - offset.min(0.) as f64;
        let position_wrtie = (pos * music_sample_rate as f64).ceil() as usize * 2;
        let position_read = (offset.max(0.) as f64 * music_sample_rate as f64).ceil() as usize * 2;
        let len = (music.len() - position_read).min(output_music_len - position_wrtie);
        let clip = music.slice(s![position_read..position_read + len]);
        let mut slice = output_music.slice_mut(s![position_wrtie..position_wrtie + len]);
        slice += &clip;
        info!("Process Music Time: {:.2?}", music_time.elapsed());
    }

    type HitSoundMap = HashMap<String, Array1<f32>>;
    let mut extra_sfxs: HitSoundMap = HitSoundMap::new();

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
        let length = chart_length as f32;
        let mut hit_fx_list: Vec<(f64, &Array1<f32>)> = Vec::new();

        if config.audio_mix_optimization {
            chart.lines.iter().flat_map(|line| &line.notes).filter(|note| !note.fake && note.time < length).for_each(|note| {
                if let Some(sfx) = get_hitsound(&note) {
                    hit_fx_list.push((before_time + note.time as f64 + judge_offset, sfx));
                }
            });
            let len = hit_fx_list.len();

            hit_fx_list.sort_by(|(a1, b1), (a2, b2)| {
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

            let mut kept = Vec::with_capacity(hit_fx_list.len());
            let mut last_arr: Option<&Array1<f32>> = None;
            let mut last_t = 0.0;
            let mut count = 0;
            let mut offset = 0.0;

            for &(pos, clip) in &hit_fx_list {
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
                    offset = rand::gen_range(0.000, 0.002);
                    kept.push((pos + offset, clip));
                } else {
                    count += 1;
                    if count <= 3 {
                        kept.push((pos + offset, clip));
                    }
                }
            }

            hit_fx_list = kept;
            let num = hit_fx_list.len();
            for (pos, sfx) in hit_fx_list {
                place_fx(pos, sfx);
            }

            let elapsed = sfx_time.elapsed();
            info!("Process Hit Effects Time: {:.2?} Equivalent Speed: {:.2} notes/sec Speed: {:.2} notes/sec", elapsed, len as f32 / elapsed.as_secs_f32(), num as f32 / elapsed.as_secs_f32())
        } else {
            let mut num = 0;
            chart.lines.iter().flat_map(|line| &line.notes).filter(|note| !note.fake && note.time < length).for_each(|note| {
                if let Some(sfx) = get_hitsound(&note) {
                    place_fx(before_time + note.time as f64 + judge_offset, sfx);
                    num += 1;
                }
            });

            let elapsed = sfx_time.elapsed();
            info!("Process Hit Effects Time: {:.2?} Speed: {:.2} notes/sec", elapsed, num as f32 / elapsed.as_secs_f32())
        }
    }

    let output_music_temp = NamedTempFile::new()?;
    let output_fx_temp = NamedTempFile::new()?;

    {
        let output_audio_time = Instant::now();

        let mut proc = cmd_hidden(&ffmpeg)
            .args(
                format!(
                    "-y -f f32le -ar {} -ac 2 -i pipe:0 -ss {} -c:a pcm_f32le -f wav",
                    music_sample_rate, video_cut_time
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
                    "-y -f f32le -ar {} -ac 2 -i pipe:0 -ss {} -c:a pcm_f32le -f wav",
                    sample_rate, video_cut_time
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
    let mut main = Main::new(
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
                    MSAA.store(true, AtomicOrdering::SeqCst);
                    Some(mst.input())
                } else {
                    MSAA.store(false, AtomicOrdering::SeqCst);
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
        (chart_length + 0.5 + GameScene::WAIT_AFTER_TIME as f64 + EndingScene::BPM_WAIT_TIME - video_cut_time) * 1000.;
    let delay_ending = format!("{}|{}", delay_ending, delay_ending);

    let ffmpeg_audio_filter_music = if config.loudness_equalization { format!(
        "[1:a]loudnorm=I=-16:LRA=24:TP=-1,aresample={}:resampler=swr,volume={}[a1];", sample_rate, volume_music,
    )} else { format!(
        "[1:a]aresample={}:resampler=swr,volume={}[a1];", sample_rate, volume_music,
    )};

    let ffmpeg_audio_filter_fx = if config.force_limit {
        format!(
            "[2:a]volume={},alimiter=limit={}:level=false:attack=0.1:release=1[a2];",
            volume_sfx, config.limit_threshold
        )
    } else if config.compression_ratio > 1. {
        format!(
            "[2:a]volume={},acompressor=threshold=0dB:ratio={}:attack=0.01:release=0.01[a2];",
            volume_sfx, config.compression_ratio
        )
    } else {
        format!("[2:a]volume={}[a2];", volume_sfx)
    };

    let ffmpeg_audio_filter_ending =
        format!("[3:a]volume={},adelay={}[a3];", volume_music, delay_ending);

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
        "-c:a {} -c:v {} -pix_fmt yuv420p {} {} {} {} -filter_complex {} -map 0:v:0 -map [a] -vf vflip -f {}",
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
        if config.hires { "mov" } else { "mp4" }
    );

    info!(
        "Preparing Render Time: {:.2?}",
        preparing_render_time.elapsed()
    );

    info!("Command: {} {} {} {} {} {} {} {} {} {}",
        &ffmpeg,
        args,
        "-i",
        output_music_temp.path().display(),
        "-i", output_fx_temp.path().display(),
        "-i", ASSET_PATH.get().unwrap().join("ending.ogg").display(),
        args2,
        output_path.display()
    );

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
        send(IPCEvent::StartRender(frames));
    }
    let render_time = Instant::now();

    let fps = fps as f64;
    let frames10 = frames / 10;
    let frames = if config.disable_loading {
        frames + ((video_cut_time + GameScene::BEFORE_DURATION as f64) * fps) as u64
    } else {
        frames
    };
    let mut step_time = Instant::now();
    let mut writed_frames: u64 = 0;
    for frame in 0..frames {
        let now = (frame as f64) / fps;
        *my_time.borrow_mut() = now.max(0.);
        gl.quad_gl.render_pass(Some(mst.output().render_pass));
        main.update()?;
        if config.disable_loading && now < video_cut_time + GameScene::BEFORE_DURATION as f64 {
            continue;
        }
        main.render(&mut painter)?;
        if *my_time.borrow() <= LoadingScene::TOTAL_TIME as f64 && !config.disable_loading {
            draw_rectangle(0., 0., 0., 0., Color::default());
        }
        gl.flush();

        if MSAA.load(AtomicOrdering::SeqCst) {
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
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[writed_frames as usize % N]);
            glReadPixels(
                0,
                0,
                vw as _,
                vh as _,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                std::ptr::null_mut(),
            );
            writed_frames += 1;
            if writed_frames >= N as u64 {
                glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[(writed_frames) as usize % N]);
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
