use std::{
    fs::{self, File},
    path::Path,
};

use dialoguer::Completion;

pub fn unzip_submission(zip_path: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let zip_file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(zip_file)?;

    let out_dir = zip_path.trim_end_matches(".zip");
    std::fs::create_dir_all(out_dir)?;

    let rg = regex::Regex::new(r"[qQ]\d+")?;

    let mut res = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = Path::new(file.name());

        if file_path.components().any(|c| c.as_os_str() == "__MACOSX") {
            continue;
        }

        if file.is_dir() {
            continue;
        }

        let file_name = Path::new(file.name()).file_name();

        if let Some(name) = file_name {
            let name = Path::new(name);
            if name.extension().and_then(|s| s.to_str()) != Some("c") {
                continue;
            }

            let name_os = name.to_str().unwrap();

            let q_number = rg.find(name_os).map_or("unknown-number", |m| m.as_str());

            let zip_name = Path::new(zip_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");

            let new_name = format!("{}_{}.c", q_number.to_lowercase(), zip_name.to_lowercase());

            let out_path = Path::new(out_dir).join(new_name);
            let mut out_file = File::create(&out_path)?;
            std::io::copy(&mut file, &mut out_file)?;
            res += 1;
        }
    }

    std::fs::remove_file(zip_path)?;

    Ok(res)
}

#[derive(Default)]
pub struct FilePathCompleter {}

impl Completion for FilePathCompleter {
    fn get(&self, input: &str) -> Option<String> {
        let path = Path::new(input);

        let dir = if path.is_dir() {
            path
        } else {
            path.parent().unwrap_or_else(|| Path::new("."))
        };

        if let Ok(entries) = fs::read_dir(dir) {
            let mut matches = Vec::new();

            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with(path.file_name().and_then(|s| s.to_str()).unwrap_or("")) {
                        matches.push(name.to_string());
                    }
                }
            }

            if matches.len() == 1 {
                return Some(dir.join(&matches[0]).to_string_lossy().into_owned());
            }
        }

        None
    }
}
