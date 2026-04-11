//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod icon;

use icon::{BIG_ICON, ICON, SMALL_ICON};
use macroquad::miniquad::conf::Icon;

pub fn build_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Phi Recorder".to_string(),
        window_width: 1280,
        window_height: 720,
        icon: Some(Icon {
            medium: ICON,
            big: BIG_ICON,
            small: SMALL_ICON,
        }),
        headless: std::env::args().len() <= 1 || matches!(
            std::env::args().skip(1).next().as_deref(),
            Some("render") | Some("--render")
        ),
        ..Default::default()
    }
}

#[macroquad::main(build_conf)]
async fn main() -> Result<(), anyhow::Error> {
    //#[allow(unused_must_use)]
    phi_recorder_lib::run().await?;
    Ok(())
}
