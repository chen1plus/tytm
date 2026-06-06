use crate::{env, fsx, manifest::Manifest};
use colored::Colorize;

pub fn entry(theme: String, sub: Option<Vec<String>>) -> anyhow::Result<()> {
    let mut manifest = Manifest::load()?;
    let themes_dir = env::TYPORA_THEME.as_path();

    if !themes_dir.exists() {
        return Err(anyhow::anyhow!("Typora themes directory does not exist."));
    }

    // Look up the theme in the manifest (by key or by CSS filename)
    let target_theme_key = if manifest.themes.contains_key(&theme) {
        Some(theme.clone())
    } else {
        let css_filename = format!("{}.css", theme);
        manifest.themes
            .iter()
            .find(|(_, entry)| entry.files.contains(&css_filename))
            .map(|(k, _)| k.clone())
    };

    if let Some(pkg_name) = target_theme_key {
        let mut entry = manifest.themes.remove(&pkg_name).unwrap();
        println!("{}", format!("Removing managed theme '{}'...", pkg_name).cyan());

        if let Some(sub_files) = sub {
            // Remove only specific files/sub-packages
            let mut remaining_files = entry.files.clone();
            for file_to_remove in sub_files {
                let full_path = themes_dir.join(&file_to_remove);
                if full_path.exists() {
                    println!("  Deleting: {}", file_to_remove);
                    fsx::Obj::from(full_path).remove()?;
                    remaining_files.retain(|f| f != &file_to_remove);
                } else {
                    println!("  {} does not exist, skipping.", file_to_remove.yellow());
                }
            }
            if !remaining_files.is_empty() {
                // If there are still files left, put the entry back in the manifest
                entry.files = remaining_files;
                manifest.add_theme(entry);
            }
        } else {
            // Remove all files belonging to this theme
            for rel_file in entry.files {
                let full_path = themes_dir.join(&rel_file);
                if full_path.exists() {
                    println!("  Deleting: {}", rel_file);
                    fsx::Obj::from(full_path).remove()?;
                }
            }
        }
        manifest.save()?;
        println!("{}", format!("Theme '{}' removed successfully.", theme).green().bold());
    } else {
        // Fall back to manual/convention deletion
        let css_file = format!("{}.css", theme);
        let folder = theme.clone();

        let css_path = themes_dir.join(&css_file);
        let folder_path = themes_dir.join(&folder);

        let css_exists = css_path.exists();
        let folder_exists = folder_path.exists();

        if !css_exists && !folder_exists {
            return Err(anyhow::anyhow!("Theme '{}' was not found.", theme));
        }

        println!("{}", format!("Theme '{}' is not tracked by manifest. Attempting convention-based removal...", theme).yellow());

        if css_exists {
            println!("  Deleting: {}", css_file);
            fsx::Obj::from(css_path).remove()?;
        }
        if folder_exists {
            println!("  Deleting: {}", folder);
            fsx::Obj::from(folder_path).remove()?;
        }

        println!("{}", format!("Theme '{}' removed successfully.", theme).green().bold());
    }

    Ok(())
}
