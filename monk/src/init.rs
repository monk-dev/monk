use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use monk_types::config::MonkConfig;
use tracing::info;

pub fn get_or_create_config(config_folder: Option<&Path>) -> anyhow::Result<MonkConfig> {
    let config = config_from_path(config_folder)?;
    info!(path=%config.data_dir.display(), "data dir");

    Ok(config)
}

#[tracing::instrument]
fn config_from_path(config_folder: Option<&Path>) -> anyhow::Result<MonkConfig> {
    let config_path = config_file_path(config_folder)?;
    info!(config_path=%config_path.display(), "config file path");

    if !config_path.exists() {
        info!("creating default config");

        let default_config = create_default_config()?;

        std::fs::create_dir_all(config_path.parent().unwrap())?;
        std::fs::write(config_path, serde_yaml::to_string(&default_config)?)?;

        Ok(default_config)
    } else {
        info!("reading config from fs");
        let config_str = std::fs::read_to_string(config_path)?;

        Ok(serde_yaml::from_str(&config_str)?)
    }
}

#[tracing::instrument(skip_all)]
pub fn config_file_path(config_folder: Option<impl AsRef<Path>>) -> anyhow::Result<PathBuf> {
    let config_folder = if let Some(path) = config_folder {
        path.as_ref().into()
    } else {
        config_dir_from_env()?
    };

    info!(path=%config_folder.display(), "config folder");
    let config_path = config_folder.join("monk.yml");
    Ok(if config_path.exists() {
        config_path
    } else {
        config_folder.join("monk.yaml")
    })
}

#[tracing::instrument]
fn config_dir_from_env() -> anyhow::Result<PathBuf> {
    if let Ok(path) = std::env::var("MONK_CONFIG_DIR") {
        info!("using MONK_CONFIG_DIR");
        return Ok(path.into());
    }

    if let Some(proj_dirs) = ProjectDirs::from("com", "Monk", "Monk") {
        info!(path = %proj_dirs.config_dir().display(), "using project dirs path");
        return Ok(proj_dirs.config_dir().into());
    }

    anyhow::bail!("Could not locate monk config dir. Please set `MONK_CONFIG_DIR`");
}

fn create_default_config() -> anyhow::Result<MonkConfig> {
    let mut config = MonkConfig::default();
    let data_dir = data_dir_from_env()?;

    std::fs::create_dir_all(&data_dir)?;

    config.data_dir = data_dir.canonicalize()?;
    Ok(config)
}

#[tracing::instrument]
fn data_dir_from_env() -> anyhow::Result<PathBuf> {
    if let Ok(path) = std::env::var("MONK_DATA_DIR") {
        info!(%path, "using MONK_DATA_DIR");
        return Ok(PathBuf::from(path));
    }

    if let Some(proj_dirs) = ProjectDirs::from("com", "Monk", "Monk") {
        info!(path = %proj_dirs.data_dir().display(), "using project dirs path");
        return Ok(proj_dirs.data_dir().into());
    }

    anyhow::bail!("Could not locate monk config dir. Please set `MONK_DATA_DIR`");
}
