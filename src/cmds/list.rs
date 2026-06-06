use std::collections::HashSet;
use std::fs;
use crate::{env, manifest::Manifest};
use comfy_table::presets::UTF8_FULL;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::{Cell, Color, Table, Attribute};

pub fn entry() -> anyhow::Result<()> {
    let mut manifest = Manifest::load()?;
    let themes_dir = env::TYPORA_THEME.as_path();

    if !themes_dir.exists() {
        return Err(anyhow::anyhow!("Typora themes directory does not exist."));
    }

    // 1. Scan the actual themes folder for all *.css files
    let mut actual_css_files = Vec::new();
    for entry in fs::read_dir(themes_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension() == Some("css".as_ref()) {
            actual_css_files.push(entry);
        }
    }

    // Sort actual css files alphabetically
    actual_css_files.sort_by_key(|e| e.file_name());

    // 2. Check which managed themes are actually installed.
    // If all files of a managed theme are missing from the disk, we auto-cleanup that theme entry.
    let mut manifest_changed = false;
    let mut active_managed_packages = HashSet::new();

    let manifest_themes: Vec<String> = manifest.themes.keys().cloned().collect();
    for theme_pkg_name in manifest_themes {
        if let Some(entry) = manifest.themes.get(&theme_pkg_name) {
            let mut any_file_exists = false;
            for rel_file in &entry.files {
                let full_path = themes_dir.join(rel_file);
                if full_path.exists() {
                    any_file_exists = true;
                    break;
                }
            }
            if !any_file_exists {
                // Auto-cleanup orphan entry
                manifest.remove_theme(&theme_pkg_name);
                manifest_changed = true;
            } else {
                active_managed_packages.insert(theme_pkg_name);
            }
        }
    }

    if manifest_changed {
        manifest.save()?;
    }

    // 3. Render Table
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Status").add_attribute(Attribute::Bold),
            Cell::new("Theme Name").add_attribute(Attribute::Bold),
            Cell::new("Source Package / URL").add_attribute(Attribute::Bold),
            Cell::new("Date").add_attribute(Attribute::Bold),
        ]);

    if actual_css_files.is_empty() {
        println!("No themes found in Typora themes directory.");
        return Ok(());
    }

    for entry in actual_css_files {
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let theme_name = path.file_stem().unwrap().to_str().unwrap().to_string();

        let metadata = entry.metadata()?;
        let utc_time: chrono::DateTime<chrono::Utc> = metadata.modified()?.into();
        let local_time = utc_time.with_timezone(&chrono::Local);
        let modified_str = local_time.format("%Y-%m-%d %H:%M:%S").to_string();

        // Find if this CSS file belongs to any managed theme
        let mut belonging_pkg = None;
        for (_, pkg_entry) in &manifest.themes {
            if pkg_entry.files.contains(&file_name) {
                belonging_pkg = Some(pkg_entry);
                break;
            }
        }

        if let Some(pkg) = belonging_pkg {
            table.add_row(vec![
                Cell::new("Managed").fg(Color::Green).add_attribute(Attribute::Bold),
                Cell::new(&theme_name).add_attribute(Attribute::Bold),
                Cell::new(&pkg.source),
                Cell::new(&pkg.installed_at),
            ]);
        } else {
            table.add_row(vec![
                Cell::new("Manual").fg(Color::Yellow),
                Cell::new(&theme_name),
                Cell::new("Manual Installation").fg(Color::DarkGrey),
                Cell::new(&modified_str),
            ]);
        }
    }

    println!("\nInstalled Typora Themes:");
    println!("{table}");
    Ok(())
}
