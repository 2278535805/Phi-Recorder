use crate::{
    common::{parse_args, read_config}, render::{RenderConfig, RenderParams, build_player}
};
use anyhow::{Result};
use macroquad::prelude::*;
use phire::{
    config::{Config, Mods},
    core::init_assets,
    fs,
    scene::{show_error, GameMode, LoadingScene, NextScene, Scene},
    time::TimeManager,
    ui::{FontArc, TextPainter, Ui},
    Main,
};
use std::{cell::RefCell, collections::VecDeque, io::BufRead, ops::DerefMut, rc::Rc};

struct BaseScene(Option<NextScene>, bool, Rc<RefCell<Option<f32>>>);
impl Scene for BaseScene {
    fn on_result(&mut self, _tm: &mut TimeManager, result: Box<dyn std::any::Any>) -> Result<()> {
        match result.downcast::<Option<f32>>() {
            Ok(result_offset) => {
                if let Some(offset) = *result_offset {
                    *self.2.borrow_mut() = Some(offset);
                }
                Ok(())
            }
            Err(result_err) => match result_err.downcast::<anyhow::Error>() {
                Ok(error) => {
                    show_error(error.context("加载谱面失败"));
                    self.1 = true;
                    Ok(())
                }
                Err(_) => Ok(()),
            },
        }
    }

    fn enter(&mut self, _tm: &mut TimeManager, _target: Option<RenderTarget>) -> Result<()> {
        if self.0.is_none() && !self.1 {
            self.0 = Some(NextScene::Exit);
        }
        Ok(())
    }
    fn update(&mut self, _tm: &mut TimeManager) -> Result<()> {
        Ok(())
    }
    fn render(&mut self, _tm: &mut TimeManager, _ui: &mut Ui) -> Result<()> {
        Ok(())
    }
    fn next_scene(&mut self, _tm: &mut TimeManager) -> phire::scene::NextScene {
        self.0.take().unwrap_or_default()
    }
}

pub async fn main(cmd: bool, tweak_offset: bool, autoplay: bool) -> Result<()> {
    init_assets();
    let (fs, config, info) = if cmd {
        let (args_input, _, args_config, args_info) = parse_args(std::env::args().collect());

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

        (fs, config, info)
    } else {
        let mut stdin = std::io::stdin().lock();
        let stdin = &mut stdin;

        let mut line = String::new();
        stdin.read_line(&mut line)?;
        let params: RenderParams = serde_json::from_str(line.trim())?;
        let path = params.path;

        let fs = fs::fs_from_file(&path)?;

        let config = params.config;
        let info = params.info;

        (fs, config, info)
    };

    let mut prpr_config: Config = config.to_config();
    if autoplay {
        prpr_config.mods |= Mods::AUTOPLAY;
    }
    prpr_config.volume_bgm = prpr_config.volume_music;
    if let Ok(fullscreen_mode) = read_config().map(|config| config.fullscreen_mode) {
        macroquad::window::set_fullscreen(fullscreen_mode);
    }

    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;
    let mut painter = TextPainter::new(font);

    let player = build_player(&config).await?;

    let tm = TimeManager::default();
    let ctm = TimeManager::from_config(&prpr_config); // strange variable name...
    let offset = Rc::new(RefCell::new(None));
    let mut main = Main::new(
        Box::new(BaseScene(
            Some(NextScene::Overlay(Box::new(
                LoadingScene::new(
                    None,
                    if tweak_offset {
                        GameMode::TweakOffset
                    } else {
                        GameMode::Exercise
                    },
                    info,
                    &prpr_config,
                    fs,
                    Some(player),
                    None,
                    None,
                )
                .await?,
            ))),
            false,
            Rc::clone(&offset),
        )),
        ctm,
        None,
    )
    .await?;

    let mut frame_times: VecDeque<(f64, u32)> = VecDeque::new(); // (time, fps)
    let mut fps_last_update_sec: u32 = 0;

    'app: loop {
        let frame_start = tm.real_time();

        main.update()?;
        main.render(&mut painter)?;
        if main.should_exit() {
            break 'app;
        }

        let frame_end = tm.real_time();
        let now_fps = (1. / (frame_end - frame_start)) as u32;
        frame_times.push_back((frame_end, now_fps));
        while frame_times.front().is_some_and(|it| frame_end - it.0 > 1.0) {
            frame_times.pop_front();
        }

        next_frame().await;
        let flash_end = tm.real_time();

        let fps_now_sec = frame_start as u32;
        if fps_last_update_sec != fps_now_sec {
            fps_last_update_sec = fps_now_sec;
            let real_fps = frame_times.len() as u32;
            let real_now_fps = (1. / (flash_end - frame_start)) as u32;
            let avg_fps = frame_times.iter().map(|(_, fps)| fps).sum::<u32>() / real_fps;
            let min_fps = frame_times.iter().map(|(_, fps)| fps).min().unwrap_or(&0);
            info!("| AVG: {}|{} NOW: {}|{}, MIN: {}", real_fps, avg_fps, real_now_fps, now_fps, min_fps);
        }
    }

    if tweak_offset {
        if let Some(result_offset) = *offset.borrow() {
            let result_json = serde_json::to_string(&result_offset)?;
            println!("{}", result_json);
        }
    }

    Ok(())
}
