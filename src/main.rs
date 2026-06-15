use anyhow::anyhow;
use clap::{Parser, Subcommand};
use std::fs;

mod fsx;
mod manifest;
mod source;

use crate::fsx::{TYPORA_THEME, TYTM_INSTALLED};
use crate::manifest::{Manifest, TYTM_REGISTRY};

#[derive(Parser)]
#[command(author = "Aaron Chen", name = "TyTM", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // /// Update the registry and installed themes
    // Update,
    /// Add a new theme
    Add {
        /// The theme ID or URL
        theme: String,
        // /// Use URL instead of ID
        // #[arg(long)]
        // url: bool,
    },

    /// List all installed themes
    #[command(alias = "ls")]
    List,

    /// Remove a theme
    #[command(alias = "rm")]
    Remove {
        /// The theme ID
        theme: String,
        // /// The sub-packages to remove
        // #[arg(short, long)]
        // sub: Option<Vec<String>>,
    },
}

// fn is_url_or_path(s: &str) -> bool {
//     s.starts_with("http://")
//         || s.starts_with("https://")
//         || s.starts_with("git@")
//         || s.starts_with("git://")
//         || s.ends_with(".git")
//         || s.ends_with(".zip")
//         || s.contains('/')
//         || s.contains('\\')
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fs::create_dir_all(TYPORA_THEME.join("tytm"))?;
    let mut installed = Manifest::load(&TYTM_INSTALLED)?;

    let cli = Cli::parse();
    match cli.command {
        Commands::Add { theme } => {
            if installed.0.keys().any(|x| x == &theme) {
                return Err(anyhow!("The theme was already installed."));
            }

            let t = TYTM_REGISTRY.0.get(&theme);
            let t = t.ok_or(anyhow!("Could not find a theme with the provided ID."))?;

            t.source.install(&t.variants).await?;
            installed.0.insert(theme, t.clone());
            installed.save(&TYTM_INSTALLED)?;
        }

        Commands::List => {
            for (_, theme) in installed.0.iter() {
                println!("{}", theme.name);
            }
        }

        Commands::Remove { theme } => {
            let t = installed.0.get(&theme);
            let t = t.ok_or(anyhow!("Could not find a theme with the provided ID."))?;

            t.remove_all()?;
            installed.0.remove(&theme);
            installed.save(&TYTM_INSTALLED)?;
        }
    }
    // match cli.command {
    //     Commands::Update => {
    //         cmds::update::entry()?;
    //     }

    //     Commands::Add {
    //         theme,
    //         url,
    //         url_type,
    //     } => {
    //         let (resolved_url, resolved_type) = if let Some(u) = url {
    //             let t = match url_type {
    //                 Some(t) => t,
    //                 None => {
    //                     if u.ends_with(".git") {
    //                         cmds::add::UrlType::Git
    //                     } else if u.ends_with(".zip") {
    //                         cmds::add::UrlType::Zip
    //                     } else {
    //                         return Err(anyhow::anyhow!("Failed to determine the url type"));
    //                     }
    //                 }
    //             };
    //             (u, t)
    //         } else {
    //             let theme_str = theme.unwrap();
    //             if is_url_or_path(&theme_str) {
    //                 let t = match url_type {
    //                     Some(t) => t,
    //                     None => {
    //                         if theme_str.ends_with(".git") {
    //                             cmds::add::UrlType::Git
    //                         } else if theme_str.ends_with(".zip") {
    //                             cmds::add::UrlType::Zip
    //                         } else {
    //                             return Err(anyhow::anyhow!("Failed to determine the url type"));
    //                         }
    //                     }
    //                 };
    //                 (theme_str, t)
    //             } else {
    //                 let registry = registry::Registry::load()?;
    //                 if let Some(entry) = registry.themes.get(&theme_str) {
    //                     let t = match url_type {
    //                         Some(t) => t,
    //                         None => match entry.url_type.as_str() {
    //                             "git" => cmds::add::UrlType::Git,
    //                             "zip" => cmds::add::UrlType::Zip,
    //                             _ => return Err(anyhow::anyhow!("Unknown url type in registry")),
    //                         },
    //                     };
    //                     (entry.url.clone(), t)
    //                 } else {
    //                     return Err(anyhow::anyhow!(
    //                         "Theme '{}' not found in registry",
    //                         theme_str
    //                     ));
    //                 }
    //             }
    //         };
    //         cmds::add::entry(&resolved_url, resolved_type)?;
    //     }

    //     Commands::Remove { theme, sub } => {
    //         cmds::remove::entry(theme, sub)?;
    //     }

    //     Commands::List => {
    //         cmds::list::entry()?;
    //     }
    // }

    Ok(())
}
