use std::fs;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use tempfile::{tempdir, tempfile};
use zip::ZipArchive;

use crate::{env, fsx};
use crate::manifest::{Manifest, ThemeEntry};
use colored::Colorize;

#[derive(Clone, Copy, ValueEnum)]
pub enum UrlType {
    Git,
    Zip,
}

pub fn entry(url: &str, url_type: UrlType) -> anyhow::Result<()> {
    let tmp_dir = tempdir()?;

    let version = match url_type {
        UrlType::Git => {
            println!("{}", "Cloning theme repository...".cyan());
            let repo = git2::Repository::clone(url, &tmp_dir)?;
            let head = repo.head()?;
            let commit = head.peel_to_commit()?;
            Some(commit.id().to_string())
        }

        UrlType::Zip => {
            let mut tmp_file = tempfile()?;

            println!("{}", "Downloading theme zip...".cyan());
            let mut response = reqwest::blocking::get(url)?;
            let headers = response.headers().clone();
            response.copy_to(&mut tmp_file)?;

            println!("{}", "Extracting zip archive...".cyan());
            ZipArchive::new(tmp_file)?.extract(&tmp_dir)?;

            headers.get(reqwest::header::ETAG)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim_matches('"').to_string())
                .or_else(|| {
                    headers.get(reqwest::header::LAST_MODIFIED)
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string())
                })
        }
    };

    let base = find_base_dir(tmp_dir.path())?;
    let mut copied_files = Vec::new();

    for entry in fs::read_dir(&base)? {
        let path = entry?.path();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();

        // this unwrap is safe because we know the path exists
        if file_name == ".git" {
            continue;
        }

        if path.is_dir() || path.extension() == Some("css".as_ref()) {
            fsx::Obj::from(path).move_to(env::TYPORA_THEME.as_path())?;
            copied_files.push(file_name);
        }
    }

    if copied_files.is_empty() {
        return Err(anyhow::anyhow!("No theme files (.css or directories) were found to install."));
    }

    // Determine theme name from primary css file
    let mut primary_css = None;
    for file in &copied_files {
        if file.ends_with(".css") {
            primary_css = Some(file.strip_suffix(".css").unwrap().to_string());
            break;
        }
    }

    let theme_name = if let Some(css_name) = primary_css {
        css_name
    } else {
        // Fallback to URL's repository/file name
        url.split('/')
            .last()
            .unwrap_or("unknown")
            .trim_end_matches(".git")
            .trim_end_matches(".zip")
            .to_string()
    };

    // Save to manifest
    let mut manifest = Manifest::load()?;
    let entry = ThemeEntry {
        name: theme_name.clone(),
        source: url.to_string(),
        version,
        installed_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        files: copied_files,
    };
    manifest.add_theme(entry);
    manifest.save()?;

    println!("{}", format!("Theme '{}' successfully installed!", theme_name).green().bold());
    Ok(())
}

fn find_base_dir(from: &Path) -> anyhow::Result<PathBuf> {
    use std::cmp::Ordering;

    let mut files = fs::read_dir(&from)?.collect::<Result<Vec<_>, _>>()?;

    files.sort_by(|a, _| match a.path().is_file() {
        true => Ordering::Less,
        false => Ordering::Greater,
    });

    for f in files {
        match f.file_type()? {
            x if x.is_file() => {
                if let Some(ext) = f.path().extension() {
                    if ext.to_str() == Some("css") {
                        return Ok(f.path().parent().unwrap().to_owned());
                    }
                }
            }
            x if x.is_dir() => {
                if let Ok(res) = find_base_dir(&f.path()) {
                    return Ok(res);
                }
            }
            _ => (),
        }
    }

    Err(anyhow::anyhow!(
        "Unable to locate the base directory from {from:?}"
    ))
}
