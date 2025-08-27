use std::{fs::File, path::Path};

pub fn unzip_submission(zip_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let zip_file = File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(zip_file)?;

    let out_dir = zip_path.trim_end_matches(".zip");
    std::fs::create_dir_all(out_dir)?;

    let rg = regex::Regex::new(r"[qQ]\d+")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

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
        }
    }

    std::fs::remove_file(zip_path)?;

    Ok(())
}
