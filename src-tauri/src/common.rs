use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
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
    let dir = if let Some(set_output_dir) = read_config()?.output_dir {
        set_output_dir
    } else {
        DATA_DIR.get().unwrap().join("output")
    };
    if dir.exists() {
        if !dir.is_dir() {
            bail!("output directory is not a directory");
        }
    } else {
        std::fs::create_dir(&dir)?;
    }
    Ok(dir)
}

pub fn test_output_dir(dir: PathBuf) -> Result<()> {
    //let dir = DATA_DIR.get().unwrap().clone();
    if dir.exists() {
        if !dir.is_dir() {
            bail!("output directory is not a directory");
        }
    } else {
        std::fs::create_dir(&dir)?;
    }
    Ok(())
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub rpe_dir: Option<PathBuf>,
    pub output_dir: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpe_dir: None,
            output_dir: None,
        }
    }
}

pub fn get_config_file() -> Result<PathBuf> {
    let file = CONFIG_DIR.get().unwrap().join("config.toml");
    if file.exists() && !file.is_file() {
        bail!("presets.toml is not a file");
    }
    Ok(file)
}

pub fn read_config() -> Result<Config> {
    let file = get_config_file()?;
    if file.exists() {
        let config: Config = toml::from_str(&std::fs::read_to_string(file)?)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

pub fn save_config(config: Config) -> Result<()> {
    let file = get_config_file()?;
    let string = toml::to_string(&config)?;
    std::fs::write(file, string)?;
    Ok(())
}

pub fn get_rpe_dir() -> Result<PathBuf> {
    if let Some(dir) = read_config()?.rpe_dir {
        if dir.exists() {
            Ok(dir)
        } else {
            bail!("rpe_dir does not exist");
        }
    } else {
        bail!("rpe_dir is not set");
    }
}

pub fn set_rpe_dir(set_dir: Option<PathBuf>) -> Result<()> {
    let mut config = read_config()?;
    config.rpe_dir = set_dir;
    save_config(config)?;
    Ok(())
}