//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use phi_recorder_lib::{build_conf, run};

#[macroquad::main(build_conf)]
async fn main() -> Result<(), anyhow::Error> {
    //#[allow(unused_must_use)]
    run().await?;
    Ok(())
}
