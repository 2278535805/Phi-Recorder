//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


fn main() -> Result<(), anyhow::Error> {
    //#[allow(unused_must_use)]
    phi_recorder_lib::run()
}
