use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum IPCEvent {
    Loading,
    Mixing,
    MixingSfx(u64),
    Sfx(u64),
    RenderFrame(u64),
    Frame(u64),
    Paused,
    Resumed,
    Done(f64),
}

pub mod client {
    use serde::Serialize;

    pub fn send<T: Serialize>(value: T) {
        println!("{}", serde_json::to_string(&value).unwrap());
    }
}
