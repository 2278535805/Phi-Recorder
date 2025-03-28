// Prevents additional console window on Windows in release, DO NOT REMOVE!!
prpr::tl_file!("render");

use crate::common::{ensure_dir, let_output_dir, output_dir, DATA_DIR};
use chrono::Local;
use anyhow::{bail, Context, Result};
use macroquad::{miniquad::gl::GLuint, prelude::*};
use prpr::{
    config::{ChallengeModeColor, Config, Mods},
    core::{init_assets, internal_id, MSRenderTarget, HitSound, Note},
    fs,
    info::ChartInfo,
    scene::{BasicPlayer, GameMode, GameScene, LoadingScene, EndingScene},
    time::TimeManager,
    ui::{FontArc, TextPainter},
    Main,
};
use sasa::AudioClip;
use serde::{Deserialize, Serialize};
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

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct RenderConfig {
    resolution: (u32, u32),
    ffmpeg_preset: String,
    ending_length: f64,
    disable_loading: bool,
    hires: bool,
    chart_debug: bool,
    chart_ratio: f32,
    all_good: bool,
    all_bad: bool,
    fps: u32,
    hardware_accel: bool,
    hevc: bool,
    mpeg4: bool,
    custom_encoder: Option<String>,
    bitrate_control: String,
    bitrate: String,

    aggressive: bool,
    challenge_color: ChallengeModeColor,
    challenge_rank: u32,
    disable_effect: bool,
    double_hint: bool,
    fxaa: bool,
    note_scale: f32,
    //offset: f32,
    particle: bool,
    player_avatar: Option<String>,
    player_name: String,
    player_rks: f32,
    sample_count: u32,
    res_pack_path: Option<String>,
    speed: f32,
    volume_music: f32,
    volume_sfx: f32,
    compression_ratio: f32,
    force_limit: bool,
    limit_threshold: f32,
    watermark: String,
    roman: bool,
    chinese: bool,
    combo: String,
    difficulty: String,
    judge_offset: f32,
    simple_file_name: bool,

    render_line: bool,
    render_line_extra: bool,
    render_note: bool,
    render_ui_pause: bool,
    render_ui_score: bool,
    render_ui_combo: bool,
    render_ui_bar: bool,
    render_bg: bool,

    max_particles: usize,

    fade: f32,
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
            chart_debug: self.chart_debug,
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
            render_ui_score: self.render_ui_score,
            render_ui_combo: self.render_ui_combo,
            render_ui_bar: self.render_ui_bar,
            render_bg: self.render_bg,

            max_particles: self.max_particles,
            fade: self.fade,
            ..Default::default()
        }
    }

    pub fn default() -> RenderConfig {
        RenderConfig {
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
            bitrate_control: "CRF".to_string(),
            bitrate: "1000k".to_string(),
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
            compression_ratio: 100.,
            force_limit: false,
            limit_threshold: 1.0,
            chart_debug: false,
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
            render_ui_score: true,
            render_ui_combo: true,
            render_ui_bar: true,
            render_bg: true,

            max_particles: 100000,
            fade: 0.0,
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
    eprintln!("Failed to find global ffmpeg. Using bundled ffmpeg");
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

pub async fn main(cmd: bool) -> Result<()> {
    let loading_time = Instant::now();

    let (mut fs, output_path, config, info) = 
    if cmd {
        init_assets();

        #[cfg(target_os = "windows")]
        {
            let app_data_dir = std::env::var("APPDATA").unwrap();
            let data_dir = PathBuf::from(app_data_dir).join("com.hlmc.phi.recorder");
            DATA_DIR.set(ensure_dir(data_dir.clone())).unwrap();
        }

        #[cfg(not(target_os = "windows"))]
        {
            DATA_DIR
                .set(ensure_dir(std::env::current_dir().unwrap().to_owned()))
                .unwrap();
        }

        let config = match (|| -> Result<RenderConfig> {
            Ok(serde_yaml::from_str(
                &std::fs::read_to_string("config.yml").context("error reading config")?,
            )?)
        })() {
            Err(err) => {
                warn!("error loading config: {:?}", err);
                RenderConfig::default()
            }
            Ok(config) => config,
        };
        let path = std::env::args().nth(2).unwrap();

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
        let safe_name2: String = info
            .composer
            .chars()
            .filter(|&it| it == '-' || it == '_' || it.is_alphanumeric())
            .collect();
        let format = if config.hires { "mov" } else { "mp4" };

        let file_name = if config.simple_file_name {
            format!(
                "{safe_name}.{safe_name2}_{level}.{format}",
            )
        } else {
            format!(
                "{} {safe_name}_{level}.{format}",
                Local::now().format("%Y-%m-%d %H-%M-%S")
            )
        };
        let output_path = if std::env::args().len() > 3 {
            let dir = std::env::args().nth(3).unwrap();
            let output_dir = PathBuf::from(dir);
            info!("output dir: {:?}", output_dir);
            let_output_dir(output_dir)?.join(file_name)
        } else {
            output_dir()?.join(file_name)
        };

        (fs, output_path, config, info)
    }
    else {
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
    let mut prpr_config = config.to_config();
    prpr_config.mods = Mods::AUTOPLAY;
    prpr_config.disable_audio = true;
    let Some(ffmpeg) = find_ffmpeg()? else {
        bail!("FFmpeg not found")
    };
    info!("ffmpeg: {}", &ffmpeg);

    let (mut chart, ..) = GameScene::load_chart(fs.deref_mut(), &info)
        .await
        .with_context(|| tl!("load-chart-failed"))?;
    macro_rules! ld {
            ($path:literal) => {
                AudioClip::new(load_file($path).await?)
                    .with_context(|| tl!("load-sfx-failed", "name" => $path))?
            };
        }
    let music: Result<_> = async { AudioClip::new(fs.load_file(&info.music).await?) }.await;
    let music = music.with_context(|| tl!("load-music-failed"))?;
    let music_length = music.length() as f64;
    let ending_music = ld!("ending.ogg");
    let sfx_click = ld!("click.ogg");
    let sfx_drag = ld!("drag.ogg");
    let sfx_flick = ld!("flick.ogg");

    let mut gl = unsafe { get_internal_gl() };

    let volume_music = config.volume_music;
    let volume_sfx = config.volume_sfx;

    let before_time: f64 = if config.disable_loading {
        GameScene::BEFORE_DURATION as f64
    } else {
        LoadingScene::TOTAL_TIME as f64 + GameScene::BEFORE_DURATION as f64
    };
    let fade_out_time: f64 = -0.5;

    let offset = chart.offset + info.offset;
    let chart_length = before_time + music_length - offset as f64 + 1.;
    let video_length = chart_length + fade_out_time + config.ending_length;

    let encoder_list =  if config.hevc {
        ["hevc_nvenc", "hevc_qsv", "hevc_amf", "hevc_vaapi"]
    } else {
        ["h264_nvenc", "h264_qsv", "h264_amf", "h264_vaapi"]
    };

    fn get_encoder(ffmpeg: &String, config: &RenderConfig, encoders: [&str; 4]) -> Option<String> {
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

        let test_encoder = |encoder: &str| -> bool {
            info!("Testing encoder: {}", encoder);
            let output = Command::new(&ffmpeg)
                .args(&["-f", "lavfi", "-i", "testsrc=size=1920x1080:rate=5:duration=1", "-pix_fmt", "yuv420p", "-c:v", encoder, "-f", "null", "-"])
                .arg("-loglevel")
                .arg("error")
                .arg("-hide_banner")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .output()
                .expect("Failed to test encoder");
        
            output.status.success()
        };

        for encoder in encoders {
            if test_encoder(encoder) {
                return Some(encoder.to_string());
            }
        }

        None
    }

    let ffmpeg_encoder = if let Some(ffmpeg_encoder) = get_encoder(&ffmpeg, &config, encoder_list) {
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

    let mut output_music = vec![0.0_f32; (video_length * sample_rate_f64).ceil() as usize * 2];
    let mut output_fx = vec![0.0_f32; (video_length * sample_rate_f64).ceil() as usize * 2];

    // let stereo_sfx = false; // TODO stereo sound effects
    let mut place = |pos: f64, clip: &AudioClip| {
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
        let len = ((music_length + config.ending_length) * sample_rate_f64) as usize;
        let start_index = (pos * sample_rate_f64).round() as usize * 2;
        let ratio = 1.0 / sample_rate_f64;
        let slice = &mut output_music[start_index..];
        for i in 0..len.min(slice.len() / 2) {
            let position = i as f64 * ratio + offset.max(0.) as f64;
            let frame = music.sample_f64(position).unwrap_or_default();
            slice[i * 2] += frame.0;
            slice[i * 2 + 1] += frame.1;
        }
        //ending
        let ending_wait_time: f64 = GameScene::WAIT_AFTER_TIME as f64 + EndingScene::BPM_WAIT_TIME;
        let mut pos = chart_length + ending_wait_time;
        while pos < video_length && config.ending_length > EndingScene::BPM_WAIT_TIME {
            let start_index = (pos * sample_rate_f64).round() as usize * 2;
            let slice = &mut output_music[start_index..];
            let len = (slice.len() / 2).min(ending_music.frame_count());
            for i in 0..len {
                let position = i as f64 * ratio;
                let frame = ending_music.sample_f64(position).unwrap_or_default();
                slice[i * 2] += frame.0;
                slice[i * 2 + 1] += frame.1;
            }
            pos += ending_music.frame_count() as f64 / sample_rate_f64;
        }
        info!("Process Music Time:{:.2?}", music_time.elapsed())
    }

    type AudioMap = std::collections::HashMap<String, AudioClip>;
    let mut extra_sfxs: AudioMap = AudioMap::new();

    chart.hitsounds.drain().for_each(|(name, clip)| {
        extra_sfxs.insert(name, clip);
    });

    let get_hitsound = |note: &Note| {
        match &note.hitsound {
            HitSound::None => None,
            HitSound::Click => Some(&sfx_click),
            HitSound::Flick => Some(&sfx_flick),
            HitSound::Drag => Some(&sfx_drag),
            HitSound::Custom(s) => extra_sfxs.get(s)
        }
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
                        place(before_time + note.time as f64 + judge_offset, sfx);
                    }
                }
            }
        }
        info!("Process Hit Effects Time:{:.2?}", sfx_time.elapsed())
    }

    let music_output = NamedTempFile::new()?;
    let fx_output = NamedTempFile::new()?;

    {
        

        let output_audio_time = Instant::now();

        let mut proc = cmd_hidden(&ffmpeg)
            .args(
                format!(
                    "-y -f f32le -ar {} -ac 2 -i pipe:0 -c:a pcm_f32le -f wav",
                    sample_rate
                )
                .split_whitespace(),
            )
            .arg(music_output.path())
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
            .arg(fx_output.path())
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

    let fps = config.fps;
    let frames = (video_length * fps as f64 + N as f64 - 1.).ceil() as u64;



    let ffmpeg_preset = "-preset";
    let mut ffmpeg_preset_name_list = config.ffmpeg_preset.split_whitespace();

    let ffmpeg_preset_name = if ffmpeg_encoder == encoder_list[0] {
        ffmpeg_preset_name_list.nth(1).unwrap_or(
            ffmpeg_preset_name_list.nth(0).unwrap_or("p4")
        )
    } else if ffmpeg_encoder == encoder_list[1] {
        ffmpeg_preset_name_list.nth(2).unwrap_or("medium")
    } else if ffmpeg_encoder == encoder_list[2] {
        ffmpeg_preset_name_list.nth(3).unwrap_or(
            ffmpeg_preset_name_list.nth(0).unwrap_or("balanced")
        )
    } else {
        ffmpeg_preset_name_list.nth(0).unwrap_or("medium")
    };

    let bitrate_control = 
    if config.bitrate_control.to_lowercase() == "crf" {
        if ffmpeg_encoder == encoder_list[0] && !config.mpeg4 {
            "-cq"
        } else if ffmpeg_encoder == encoder_list[1] || config.mpeg4 || ffmpeg_encoder == encoder_list[3] {
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

    let mut ffmpeg_audio_effect = if config.force_limit {
        format!("[1:a]volume={}[a1];[2:a]volume={},alimiter=limit={}:level=false:attack=0.1:release=1[a2];", config.volume_music, config.volume_sfx, config.limit_threshold)
    } else if config.compression_ratio > 1. {
        format!("[1:a]volume={}[a1];[2:a]volume={},acompressor=threshold=0dB:ratio={}:attack=0.01:release=0.01[a2];", config.volume_music, config.volume_sfx, config.compression_ratio)
    } else {
        format!("[1:a]volume={}[a1];[2:a]volume={}[a2];", config.volume_music, config.volume_sfx)
    };

    if config.hires{
        ffmpeg_audio_effect += "[a1][a2]amix=inputs=2:normalize=0[a]"
    } else {
        ffmpeg_audio_effect += "[a1][a2]amix=inputs=2:normalize=0[a3];[a3]alimiter=limit=1.0:level=false:attack=0.1:release=1[a]";
    }

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
        ffmpeg_audio_effect,
        if config.disable_loading {
            format!("-ss {}", before_time)
        } else {
            "".to_string()
        },
        if config.hires { "mov" } else { "mp4" }
    );

    info!(
        "Preparing Render Time:{:.2?}",
        preparing_render_time.elapsed()
    );
    let pre_render_time = Instant::now();

    //info!("Command: {} {} {} {} {}", "ffmpeg", args,"-", args2, output_path.display());
    let mut proc = cmd_hidden(&ffmpeg)
        .args(args.split_whitespace())
        .arg("-i")
        .arg(music_output.path())
        .arg("-i")
        .arg(fx_output.path())
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

    const N: usize = 60;
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
    for frame in 0..N {
        *my_time.borrow_mut() = (frame as f64 / fps).max(0.);
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
        unsafe {
            use miniquad::gl::*;
            //let tex = mst.output().texture.raw_miniquad_texture_handle();
            glBindFramebuffer(GL_READ_FRAMEBUFFER, internal_id(mst.output()));
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[frame]);
            glReadPixels(
                0,
                0,
                vw as _,
                vh as _,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                std::ptr::null_mut(),
            );
        }
        if ipc {
            send(IPCEvent::Frame);
        }
    }
    info!("Pre-Render Time:{:.2?}", pre_render_time.elapsed());

    let frames10 = frames / 10;
    let mut step_time = Instant::now();
    for frame in N as u64..frames {
        if frame % frames10 == 0 {
            let proc = (frame as f32 / frames as f32 * 100.).ceil() as i8 / 10 * 10;
            info!(
                "Render progress: {:.0}% Time elapsed: {:.2}s",
                proc,
                step_time.elapsed().as_secs_f32()
            );
            step_time = Instant::now();
        }
        *my_time.borrow_mut() = (frame as f64 / fps).max(0.);
        gl.quad_gl.render_pass(Some(mst.output().render_pass));
        //clear_background(BLACK);
        main.viewport = Some((0, 0, vw as _, vh as _));
        main.update()?;
        main.render(&mut painter)?;
        // TODO magic. can't remove this line.
        if *my_time.borrow() <= LoadingScene::TOTAL_TIME as f64 && !config.disable_loading {
            draw_rectangle(0., 0., 0., 0., Color::default());
        }

        gl.flush();

        if MSAA.load(Ordering::SeqCst) {
            mst.blit();
        }
        unsafe {
            use miniquad::gl::*;
            //let tex = mst.output().texture.raw_miniquad_texture_handle();
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

            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[(frame + 1) as usize % N]);
            let src = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8);
            if !src.is_null() {
                input.write_all(&std::slice::from_raw_parts(src as *const u8, byte_size))?;
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
    info!("Task done in {:.2?}", render_start_time.elapsed());
    if ipc {
        send(IPCEvent::Done(render_start_time.elapsed().as_secs_f64()));
    }
    Ok(())
}
