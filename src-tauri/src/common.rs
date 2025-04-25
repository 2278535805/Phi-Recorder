use anyhow::{bail, Result};
use std::{collections::HashMap, path::PathBuf, sync::OnceLock};

use crate::render::RenderConfig;

pub static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
pub static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn ensure_dir(path: PathBuf) -> PathBuf {
    if path.exists() {
        if !path.is_dir() {
            panic!("{} is not a directory", path.display());
        }
    } else {
        std::fs::create_dir_all(&path).unwrap();
    }
    path
}

pub fn output_dir() -> Result<PathBuf> {
    let dir = DATA_DIR.get().unwrap().join("output");
    if dir.exists() {
        if !dir.is_dir() {
            bail!("output directory is not a directory");
        }
    } else {
        std::fs::create_dir(&dir)?;
    }
    Ok(dir)
}

pub fn let_output_dir(dir: PathBuf) -> Result<PathBuf> {
    //let dir = DATA_DIR.get().unwrap().clone();
    if dir.exists() {
        if !dir.is_dir() {
            bail!("output directory is not a directory");
        }
    } else {
        std::fs::create_dir(&dir)?;
    }
    Ok(dir)
}

pub fn respack_dir() -> Result<PathBuf> {
    let dir = CONFIG_DIR.get().unwrap().join("respack");
    if dir.exists() {
        if !dir.is_dir() {
            bail!("resource pack directory is not a directory");
        }
    } else {
        std::fs::create_dir(&dir)?;
    }
    Ok(dir)
}

pub fn get_presets_toml_file() -> Result<PathBuf> {
    let file = CONFIG_DIR.get().unwrap().join("presets.toml");
    if file.exists() && !file.is_file() {
        bail!("presets.toml is not a file");
    }
    Ok(file)
}

pub fn get_presets_json_file() -> Result<PathBuf> {
    let file = CONFIG_DIR.get().unwrap().join("presets.json");
    if file.exists() && !file.is_file() {
        bail!("presets.json is not a file");
    }
    Ok(file)
}

pub async fn save_presets(presets: &HashMap<String, RenderConfig>) -> Result<()> {
    let file = get_presets_toml_file()?;
    let toml_string = toml::to_string(presets)?;
    std::fs::write(file, toml_string)?;
    Ok(())
}

pub fn get_rpe_dir() -> Result<PathBuf> {
    let file = CONFIG_DIR.get().unwrap().join("rpe_path.txt");
    Ok(file)
}

pub fn rpe_dir() -> Result<Option<PathBuf>> {
    let file = get_rpe_dir()?;
    if file.exists() {
        if !file.is_file() {
            bail!("rpe_path.txt is not a file");
        }
    } else {
        return Ok(None);
    }
    let dir = PathBuf::from(std::fs::read_to_string(file)?);
    Ok(if dir.exists() { Some(dir) } else { None })
}