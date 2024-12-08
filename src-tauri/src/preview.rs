use crate::render::{build_player, RenderParams};
use anyhow::Result;
use macroquad::prelude::*;
use prpr::{
    config::{Config, Mods},
    fs,
    scene::{show_error, GameMode, LoadingScene, NextScene, Scene},
    time::TimeManager,
    ui::{FontArc, TextPainter, Ui},
    Main,
};
use std::io::BufRead;

struct BaseScene(Option<NextScene>, bool);
impl Scene for BaseScene {
    fn on_result(&mut self, _tm: &mut TimeManager, result: Box<dyn std::any::Any>) -> Result<()> {
        show_error(
            result
                .downcast::<anyhow::Error>()
                .unwrap()
                .context("加载谱面失败"),
        );
        self.1 = true;
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

pub async fn main() -> Result<()> {
    set_pc_assets_folder(&std::env::args().nth(2).unwrap());

    let mut stdin = std::io::stdin().lock();
    let stdin = &mut stdin;

    let mut line = String::new();
    stdin.read_line(&mut line)?;
    let params: RenderParams = serde_json::from_str(line.trim())?;

    let fs = fs::fs_from_file(&params.path)?;
    let info = params.info;
    let mut config: Config = params.config.to_config();
    config.mods |= Mods::AUTOPLAY;

    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;
    let mut painter = TextPainter::new(font);

    let player = build_player(&params.config).await?;

    let tm = TimeManager::default();
    let ctm = TimeManager::from_config(&config); // strange variable name...
    let mut main = Main::new(
        Box::new(BaseScene(
            Some(NextScene::Overlay(Box::new(
                LoadingScene::new(GameMode::Normal, info, &config, fs, Some(player), None, None)
                    .await?,
            ))),
            false,
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

    Ok(())
}
