use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use tracing::{warn, info};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Read, Write},
    path::PathBuf,
    sync::OnceLock,
};
use zip::{write::FileOptions, ZipWriter};

use crate::render::RenderConfig;

pub static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
pub static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn parse_args(args: Vec<String>) -> (Option<String>, Option<String>, Option<String>) {
    let mut args_input = None;
    let mut args_output = None;
    let mut args_config = None;
    let mut args_now = 1;
    while args_now < args.len() {
        match args[args_now].as_str() {
            "--output" => {
                args_output = args.get(args_now + 1).cloned();
                args_now += 2;
            }
            "--config" => {
                args_config = args.get(args_now + 1).cloned();
                args_now += 2;
            }
            arg => {
                if !arg.starts_with("--") && args_input.is_none() {
                    args_input = Some(arg.to_string());
                } else {
                    println!("Unknown argument: {}", arg);
                }
                args_now += 1;
            }
        }
    }
    return (args_input, args_output, args_config);
}

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
    pub encoder_avc: Option<String>,
    pub encoder_hevc: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rpe_dir: None,
            output_dir: None,
            encoder_avc: None,
            encoder_hevc: None,
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

#[derive(Deserialize, Serialize, Debug)]
pub struct Effect {
    pub shader: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Extra {
    pub effects: Vec<Effect>,
}

pub fn collect_chart_files(
    directory: PathBuf,
    parent: PathBuf,
) -> Result<HashMap<String, PathBuf>> {
    let mut file_list: HashMap<String, PathBuf> = HashMap::new();

    if directory.is_dir() {
        for entry in std::fs::read_dir(directory)? {
            if let Ok(entry) = entry {
                let path = entry.path();

                if path.is_file() {
                    let filename = path.file_name().unwrap().to_string_lossy();
                    let file_path = path
                        .strip_prefix(&parent)?
                        .display()
                        .to_string()
                        .replace('\\', "/");
                    if filename.starts_with("AutoSave_")
                        || filename.starts_with("blur_")
                        || filename.starts_with("blur1_")
                        || filename.starts_with("tempfile_")
                        || filename == "createTime.txt"
                    {
                        continue;
                    }
                    file_list.insert(file_path, path);
                } else if path.is_dir() {
                    file_list.extend(collect_chart_files(path, parent.clone())?);
                }
            }
        }
    }

    Ok(file_list)
}

pub async fn create_zip(output_path: PathBuf, files: HashMap<String, PathBuf>) -> Result<()> {
    let file = File::create(&output_path)?;
    let mut zip = ZipWriter::new(BufWriter::new(file));

    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(9))
        .unix_permissions(0o755);

    for (file_name, file_path) in files {
        zip.start_file(file_name, options)?;
        let mut buffer = Vec::new();
        File::open(&file_path)?.read_to_end(&mut buffer)?;
        zip.write_all(&buffer)?;
    }

    zip.finish()?;
    info!("Create ZIP: {}", output_path.display());
    Ok(())
}
