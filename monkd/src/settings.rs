use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::{Path, PathBuf};

use crate::adapter::AdapterSlug;
use crate::error::Error;
use crate::index::settings::IndexSettings;
use crate::metadata::{file_store::StoreSettings, offline_store::OfflineSettings};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    daemon: DaemonSettings,
    store: StoreSettings,
    offline: OfflineSettings,
    index: IndexSettings,
    log_dir: PathBuf,
    adapters: Vec<AdapterSlug>,
    #[serde(skip)]
    config_path: PathBuf,
}

impl Settings {
    pub fn get_settings(config_path: Option<PathBuf>) -> Result<Self, Error> {
        use config::{Config, Environment, File};
        let mut cpath = PathBuf::new();

        let config_path: Option<PathBuf> = if let Some(path) = config_path {
            Some(path.into())
        } else {
            if let Some(dirs) = crate::get_dirs() {
                use std::{fs::metadata, io::ErrorKind};

                let path = dirs.config_dir().join("monkd.yaml");

                let _meta = match metadata(&path) {
                    Ok(_) => {}
                    Err(e) if e.kind() == ErrorKind::NotFound => {
                        use std::{
                            fs::{create_dir_all, File},
                            io::Write,
                        };

                        tracing::info!("creating default config in: {}", path.display());

                        create_dir_all(&path.parent().unwrap())?;

                        // File doesn't exist, so create default settings
                        // and write it there:
                        let default = Settings::default();
                        let mut file = File::create(&path).unwrap();
                        file.write_all(serde_yaml::to_string(&default).unwrap().as_bytes())?;
                    }
                    Err(_e) => {
                        println!("error getting metadata for {}", path.display());
                        std::process::exit(1);
                    }
                };

                Some(path)
            } else {
                None
            }
        };

        let mut config = Config::default();

        if let Some(config_path) = config_path {
            config.merge(File::with_name(&config_path.to_str().unwrap()))?;
            cpath = config_path;
        }

        config.merge(Environment::with_prefix("monk"))?;

        match config.try_into::<Settings>() {
            Ok(mut settings) => {
                settings.set_config_path(cpath);
                Ok(settings)
            }
            Err(e) => Err(Error::ConfigError(e)),
        }
    }

    fn set_config_path(&mut self, path: PathBuf) {
        self.config_path = path;
    }

    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    pub fn daemon(&self) -> &DaemonSettings {
        &self.daemon
    }
    pub fn store(&self) -> &StoreSettings {
        &self.store
    }
    pub fn offline(&self) -> &OfflineSettings {
        &self.offline
    }

    pub fn index(&self) -> &IndexSettings {
        &self.index
    }

    pub fn adapters(&self) -> &[AdapterSlug] {
        &self.adapters
    }

    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

impl Default for Settings {
    fn default() -> Self {
        if let Some(dirs) = crate::get_dirs() {
            Self {
                daemon: Default::default(),
                store: Default::default(),
                offline: Default::default(),
                index: Default::default(),
                log_dir: dirs.data_dir().join("logs"),
                adapters: vec![AdapterSlug::Http],
                config_path: PathBuf::new(),
            }
        } else {
            Self {
                daemon: Default::default(),
                store: Default::default(),
                offline: Default::default(),
                index: Default::default(),
                log_dir: "./logs".into(),
                adapters: vec![AdapterSlug::Http],
                config_path: PathBuf::new(),
            }
        }
    }
}

/// The main Daemon settings. Defaults are:
/// address: 127.0.0.1:41562
/// timeout: 1000
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSettings {
    pub address: IpAddr,
    pub port: u16,
    pub timeout: usize,
    pub download_after_add: bool,
    pub download_on_open: bool,
}

impl Default for DaemonSettings {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".parse().unwrap(),
            port: 41562,
            timeout: 10000,
            download_after_add: true,
            download_on_open: true,
        }
    }
}
