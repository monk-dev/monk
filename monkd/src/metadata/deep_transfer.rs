// This file contains all the file utilities for deep imports and exports
// Files that are copied:
// store.json
// offline.json ---> need to also set all indexed status to not done
// offline/

use super::{offline_store::OfflineStore, FileStore};
use crate::error::Error;
use crate::settings::Settings;
use std::fs::File;
use std::io::copy;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use walkdir::WalkDir;
use zip::write::FileOptions;

pub fn import_deep_copy(
    file: impl AsRef<Path>,
    filestore: &mut FileStore,
    offline_store: &mut OfflineStore,
) -> Result<(), Error> {
    let f = std::fs::File::open(file)?;
    let tmp_dir = TempDir::new()?;
    let mut archive = zip::ZipArchive::new(f).unwrap();
    archive.extract(&tmp_dir)?;
    let settings = Settings::get_settings(None)?;

    let meta_file = tmp_dir.path().join("store.json");
    let import_offline_store = tmp_dir.path().join("offline.json");
    let offline_data = settings.offline().data_folder.clone();
    let import_offline_data = tmp_dir.path().join("offline");

    filestore.import_file(meta_file.to_str().unwrap_or_default().to_string())?;
    offline_store.import(import_offline_store)?;
    if let Ok(entries) = std::fs::read_dir(import_offline_data) {
        for entry in entries {
            if let Ok(entry) = entry {
                let mut to = offline_data.clone();
                to.push(entry.file_name());
                std::fs::rename(entry.path(), to)?;
            }
        }
    }
    Ok(())
}

pub fn export_deep_copy(file: PathBuf) -> Result<(), Error> {
    let out_file = std::fs::File::create(file).unwrap();
    let mut zip = zip::ZipWriter::new(BufWriter::with_capacity(65536, out_file));
    let settings = Settings::get_settings(None)?;
    let meta_file = settings.store().path.clone();
    let offline_store = settings.offline().store_file.clone();
    let offline_data = settings.offline().data_folder.clone();
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let f = File::open(&meta_file)?;
    let mut reader = BufReader::with_capacity(65536, &f);
    zip.start_file(
        meta_file
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default(),
        options,
    )?;
    copy(&mut reader, &mut zip)?;

    let f = File::open(&offline_store)?;
    let mut reader = BufReader::with_capacity(65536, &f);
    zip.start_file(
        offline_store
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default(),
        options,
    )?;
    copy(&mut reader, &mut zip)?;

    zip_dir(&offline_data, &mut zip, options)?;
    zip.finish()?;
    Ok(())
}

fn zip_dir(
    dir: &PathBuf,
    zip: &mut zip::ZipWriter<BufWriter<File>>,
    options: FileOptions,
) -> Result<(), Error> {
    let walkdir = WalkDir::new(dir);
    let it = walkdir.into_iter().filter_map(|e| e.ok());
    let prefix = dir.parent().unwrap().clone();

    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(&prefix).unwrap();
        println!("{:?}, {:?}", path, name);

        if path.is_file() {
            zip.start_file(name.to_str().unwrap_or_default(), options)?;
            let f = File::open(&path)?;
            let mut reader = BufReader::with_capacity(65536, &f);
            copy(&mut reader, zip)?;
        } else if name.as_os_str().len() != 0 {
            // do not re-add root
            zip.add_directory(name.to_str().unwrap_or_default(), options)?;
        }
    }
    Ok(())
}
