// Prevents additional console window on Windows in release, DO NOT REMOVE!!
prpr::tl_file!("render");

use anyhow::{bail, Context, Result};
use macroquad::{miniquad::gl::GLuint, prelude::*};
use prpr::{
    config::{ChallengeModeColor, Config, Mods},
    core::{internal_id, MSRenderTarget, NoteKind},
    fs,
    info::ChartInfo,
    scene::{BasicPlayer, GameMode, GameScene, LoadingScene},
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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderConfig {
    resolution: (u32, u32),
    ffmpeg_preset: String,
    ending_length: f64,
    disable_loading: bool,
    chart_debug: bool,
    chart_ratio: f32,
    fps: u32,
    hardware_accel: bool,
    bitrate_control: String,
    bitrate: String,

    aggressive: bool,
    challenge_color: ChallengeModeColor,
    challenge_rank: u32,
    disable_effect: bool,
    double_hint: bool,
    fxaa: bool,
    note_scale: f32,
    particle: bool,
    player_avatar: Option<String>,
    player_name: String,
    player_rks: f32,
    sample_count: u32,
    res_pack_path: Option<String>,
    speed: f32,
    volume_music: f32,
    volume_sfx: f32,
}

impl RenderConfig {
    pub fn to_config(&self) -> Config {
        Config {
            aggressive: self.aggressive,
            challenge_color: self.challenge_color.clone(),
            challenge_rank: self.challenge_rank,
            disable_effect: self.disable_effect,
            double_hint: self.double_hint,
            fxaa: self.fxaa,
            note_scale: self.note_scale,
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
            ..Default::default()
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

pub async fn main() -> Result<()> {
    use crate::ipc::client::*;

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

    let mut fs = fs::fs_from_file(&path)?;

    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;

    let Some(ffmpeg) = find_ffmpeg()? else {
        bail!("FFmpeg not found")
    };
    dbg!(&ffmpeg);

    let mut painter = TextPainter::new(font);

    let mut config = params.config.to_config();
    config.mods = Mods::AUTOPLAY;

    let info = params.info;

    let (chart, ..) = GameScene::load_chart(fs.deref_mut(), &info)
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
    let ending = ld!("ending.ogg");
    let track_length = music.length() as f64;
    let sfx_click = ld!("click.ogg");
    let sfx_drag = ld!("drag.ogg");
    let sfx_flick = ld!("flick.ogg");

    let mut gl = unsafe { get_internal_gl() };

    let volume_music = std::mem::take(&mut config.volume_music);
    let volume_sfx = std::mem::take(&mut config.volume_sfx);

    let length = track_length - chart.offset.min(0.) as f64 + 1.;
    let video_length = O + length + A + params.config.ending_length;
    let offset = chart.offset.max(0.);

    let render_start_time = Instant::now();

    send(IPCEvent::StartMixing);
    let mixing_output = NamedTempFile::new()?;
    let sample_rate = 48000;
    let sample_rate_f64 = sample_rate as f64;
    assert_eq!(sample_rate, ending.sample_rate());
    assert_eq!(sample_rate, sfx_click.sample_rate());
    assert_eq!(sample_rate, sfx_drag.sample_rate());
    assert_eq!(sample_rate, sfx_flick.sample_rate());
    let mut output = vec![0.0_f32; (video_length * sample_rate_f64).ceil() as usize * 2];
    {
        let pos = O - chart.offset.min(0.) as f64;
        let count = (music.length() as f64 * sample_rate_f64) as usize;
        let mut it = output[((pos * sample_rate_f64).round() as usize * 2)..].iter_mut();
        let ratio = 1. / sample_rate_f64;
        for frame in 0..count {
            let position = frame as f64 * ratio;
            let frame = music.sample(position as f32).unwrap_or_default();
            *it.next().unwrap() += frame.0 * volume_music;
            *it.next().unwrap() += frame.1 * volume_music;
        }
    }
    
    let mut place = |pos: f64, clip: &AudioClip, volume: f32| {
        let position = (pos * sample_rate_f64).round() as usize * 2;
        if position >= output.len() {
            return 0;
        }
        let slice = &mut output[position..];
        let len = (slice.len() / 2).min(clip.frame_count());
        let mut it = slice.iter_mut();
        // TODO optimize?
        for frame in clip.frames()[..len].iter() {
            let dst = it.next().unwrap();
            *dst += frame.0 * volume;
            let dst = it.next().unwrap();
            *dst += frame.1 * volume;
            /*if let (Some(dst1), Some(dst2)) = (it.next(), it.next()) {
                *dst1 += frame.0 * volume;
                *dst2 += frame.1 * volume;
            }*/
        }
        return len;
    };

    // 尝试在volume_sfx=0时不处理音效
    if volume_sfx > 0.0 {
        for note in chart
            .lines
            .iter()
            .flat_map(|it| it.notes.iter())
            .filter(|it| !it.fake)
        {
            place(
                O + note.time as f64 + offset as f64,
                match note.kind {
                    NoteKind::Click | NoteKind::Hold { .. } => &sfx_click,
                    NoteKind::Drag => &sfx_drag,
                    NoteKind::Flick => &sfx_flick,
                },
                volume_sfx,
            );
        }
    }
    let mut pos = O + length + A;
    while place(pos, &ending, volume_music) != 0 {
        pos += ending.frame_count() as f64 / sample_rate_f64;
    }
    let mut proc = cmd_hidden(&ffmpeg)
        .args(format!("-y -f f32le -ar {} -ac 2 -i - -c:a pcm_f32le -f wav", sample_rate).split_whitespace())
        .arg(mixing_output.path())
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| tl!("run-ffmpeg-failed"))?;
    let input = proc.stdin.as_mut().unwrap();
    let mut writer = BufWriter::new(input);
    for sample in output.into_iter() {
        writer.write_all(&sample.to_le_bytes())?;
    }
    drop(writer);
    proc.wait()?;

    let (vw, vh) = params.config.resolution;
    let mst = Rc::new(MSRenderTarget::new((vw, vh), config.sample_count));
    let my_time: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.));
    let tm = TimeManager::manual(Box::new({
        let my_time = Rc::clone(&my_time);
        move || *(*my_time).borrow()
    }));
    static MSAA: AtomicBool = AtomicBool::new(false);
    let player = build_player(&params.config).await?;
    let mut main = Main::new(
        Box::new(
            LoadingScene::new(GameMode::Normal, info, config, fs, Some(player), None, None).await?,
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

    const O: f64 = LoadingScene::TOTAL_TIME as f64 + GameScene::BEFORE_TIME as f64;
    const A: f64 = 0.7 + 0.3 + 0.4;

    let fps = params.config.fps;
    let frame_delta = 1. / fps as f32;

    let codecs = String::from_utf8(
        cmd_hidden(&ffmpeg)
            .arg("-codecs")
            .output()
            .with_context(|| tl!("run-ffmpeg-failed"))?
            .stdout,
    )?;
    let use_cuda = params.config.hardware_accel && codecs.contains("h264_nvenc");
    let has_qsv = params.config.hardware_accel && codecs.contains("h264_qsv");
    let has_amf = params.config.hardware_accel && codecs.contains("h264_amf");
    let ffmpeg_preset =  if !use_cuda && !has_qsv && has_amf {"-quality"} else {"-preset"};
    let ffmpeg_preset_name_list = params.config.ffmpeg_preset.split_whitespace();
    let mut ffmpeg_preset_name = if use_cuda {ffmpeg_preset_name_list.nth(1)
    } else if has_qsv {ffmpeg_preset_name_list.nth(0)
    } else if has_amf {ffmpeg_preset_name_list.nth(2)
    } else {ffmpeg_preset_name_list.nth(0)};

    let mut args = "-y -f rawvideo -c:v rawvideo".to_owned();
    if use_cuda {
        args += " -hwaccel_output_format cuda";
    }
    write!(&mut args, " -s {vw}x{vh} -r {fps} -pix_fmt rgba -i - -i")?;

    let args2 = format!(
        "-c:a copy -c:v {} -pix_fmt yuv420p {} {} {} {} -map 0:v:0 -map 1:a:0 {} -vf vflip -f mov",
        //"-c:a copy -c:v {} -pix_fmt yuv420p -b:v {} -map 0:v:0 -map 1:a:0 -vf vflip -f mp4",
        if use_cuda {"h264_nvenc"} 
        else if has_qsv {"h264_qsv"} 
        //else if has_amf {"h264_amf"}
        else if params.config.hardware_accel {bail!(tl!("no-hwacc"));} 
        else {"libx264"},
        if params.config.bitrate_control == "CRF" {
            if use_cuda {"-cq"}
            else if has_qsv {"-q"}
            //else if has_amf {"-qp_p"}
            else {"-crf"}
        } else {
            "-b:v"
        },
        params.config.bitrate,
        ffmpeg_preset,
        ffmpeg_preset_name.unwrap(),
        if params.config.disable_loading{"-ss 00:00:03.5"}
        else{"-ss 00:00:00"},
    );

    let mut proc = cmd_hidden(&ffmpeg)
        .args(args.split_whitespace())
        .arg(mixing_output.path())
        .args(args2.split_whitespace())
        .arg(output_path)
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| tl!("run-ffmpeg-failed"))?;
    let mut input = proc.stdin.take().unwrap();

    let byte_size = vw as usize * vh as usize * 4;

    let frames = (video_length / frame_delta as f64).ceil() as u64;

    const N: usize = 3;
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

    send(IPCEvent::StartRender(frames));

    for frame in 0..frames {
        *my_time.borrow_mut() = (frame as f32 * frame_delta).max(0.) as f64;
        gl.quad_gl.render_pass(Some(mst.output().render_pass));
        clear_background(BLACK);
        main.viewport = Some((0, 0, vw as _, vh as _));
        main.update()?;
        main.render(&mut painter)?;
        // TODO magic. can't remove this line.
        draw_rectangle(0., 0., 0., 0., Color::default());
        gl.flush();

        if MSAA.load(Ordering::SeqCst) {
            mst.blit();
        }
        unsafe {
            use miniquad::gl::*;
            let tex = mst.output().texture.raw_miniquad_texture_handle();
            glBindFramebuffer(GL_READ_FRAMEBUFFER, internal_id(mst.output()));

            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[frame as usize % N]);
            glReadPixels(
                0,
                0,
                tex.width as _,
                tex.height as _,
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
        send(IPCEvent::Frame);
    }
    drop(input);
    proc.wait()?;

    send(IPCEvent::Done(render_start_time.elapsed().as_secs_f64()));
    Ok(())
}
