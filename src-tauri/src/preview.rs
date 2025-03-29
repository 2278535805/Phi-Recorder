use crate::render::{build_player, RenderConfig, RenderParams};
use anyhow::{Context, Result};
use macroquad::prelude::*;
use prpr::{
    config::{Config, Mods}, core::init_assets, fs, scene::{show_error, GameMode, LoadingScene, NextScene, Scene}, time::TimeManager, ui::{FontArc, TextPainter, Ui}, Main
};
use std::{cell::RefCell, io::BufRead, ops::DerefMut, rc::Rc};

struct BaseScene(Option<NextScene>, bool, Rc<RefCell<f32>>);
impl Scene for BaseScene {
    /*fn on_result(&mut self, _tm: &mut TimeManager, result: Box<dyn std::any::Any>) -> Result<()> {
        show_error(
            result
                .downcast::<anyhow::Error>()
                .unwrap()
                .context("加载谱面失败"),
        );
        self.1 = true;
        Ok(())
    }*/

    fn on_result(&mut self, _tm: &mut TimeManager, result: Box<dyn std::any::Any>) -> Result<()> {
        let _res = match result.downcast::<Option<f32>>() {
            Ok(offset) => {
                if let Some(offset) = *offset {
                    *self.2.borrow_mut() = offset;
                }
                return Ok(());
            }
            Err(result) => result,
        };
        Ok(())
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
    fn next_scene(&mut self, _tm: &mut TimeManager) -> prpr::scene::NextScene {
        self.0.take().unwrap_or_default()
    }
}

pub async fn main(cmd: bool, tweak_offset: bool) -> Result<()> {
    let (fs, config, info) = 
    if cmd {
        init_assets();

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

        (fs, config, info)
    }
    else {
        set_pc_assets_folder(&std::env::args().nth(2).unwrap());
    
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
    if matches!(std::env::args().nth(1).as_deref(), Some("preview") | Some("--preview")) {
        prpr_config.mods |= Mods::AUTOPLAY;
    }

    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;
    let mut painter = TextPainter::new(font);

    let player = build_player(&config).await?;

    let tm = TimeManager::default();
    let ctm = TimeManager::from_config(&prpr_config); // strange variable name...
    let offset = Rc::new(RefCell::new(0.0f32));
    let mut main = Main::new(
        Box::new(BaseScene(
            Some(NextScene::Overlay(Box::new(
                LoadingScene::new(
                    if tweak_offset {
                        GameMode::TweakOffset
                    } else {
                        GameMode::Normal
                    }, 
                    info, 
                    &prpr_config, 
                    fs, 
                    Some(player), 
                    None, 
                    None
                )
                    .await?,
            ))),
            false,
            Rc::clone(&offset)
        )),
        ctm,
        None,
    )
    .await?;
    let mut fps_time = -1;

    'app: loop {
        let frame_start = tm.real_time();
        main.update()?;
        main.render(&mut painter)?;
        if main.should_exit() {
            break 'app;
        }

        let t = tm.real_time();
        let fps_now = t as i32;
        if fps_now != fps_time {
            fps_time = fps_now;
            info!("| {}", (1. / (t - frame_start)) as u32);
        }

        next_frame().await;
    }

    if tweak_offset {
        let result_offset = *offset.borrow();
        println!("{{update offset:{}}}", result_offset);
    }

    Ok(())
}