use anyhow::anyhow;
use clap::{Parser, Subcommand};
use inquire::MultiSelect;
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
    /// Add a new theme
    Add {
        /// The theme ID or URL
        theme: String,

        // /// Use URL instead of ID
        // #[arg(long)]
        // url: bool,
        /// The variants to install
        #[arg(long, short)]
        variant: Vec<String>,
    },

    /// List all installed themes
    #[command(alias = "ls")]
    List,

    /// Remove a theme
    #[command(alias = "rm")]
    Remove {
        /// The theme ID
        theme: String,

        /// The variants to install
        #[arg(long, short)]
        variant: Vec<String>,
    },

    /// Update all installed themes
    Update,

    /// Open the Typora themes folder
    Open,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fs::create_dir_all(TYPORA_THEME.join("tytm"))?;
    let mut installed = Manifest::load(&TYTM_INSTALLED)?;

    let cli = Cli::parse();
    match cli.command {
        Commands::Add {
            theme: tid,
            variant,
        } => {
            if installed.0.keys().any(|x| x == &tid) {
                return Err(anyhow!("The theme was already installed."));
            }

            let t = TYTM_REGISTRY.0.get(&tid);
            let t = t.ok_or(anyhow!("Cannot find a theme with the provided ID."))?;

            let vars: Vec<_> = if !variant.is_empty() {
                t.variants
                    .clone()
                    .into_iter()
                    .filter(|x| variant.contains(&x.name))
                    .collect()
            } else if t.variants.len() == 1 {
                t.variants.clone()
            } else {
                let options: Vec<_> = t.variants.iter().map(|v| v.name.clone()).collect();
                let selected = MultiSelect::new("Select variants to install:", options)
                    .prompt()
                    .map_err(|e| anyhow!("Variant selection cancelled: {e}"))?;
                if selected.is_empty() {
                    return Err(anyhow!("No variants selected."));
                }
                t.variants
                    .clone()
                    .into_iter()
                    .filter(|x| selected.contains(&x.name))
                    .collect()
            };

            let mut theme = t.clone();
            theme.variants = vars.clone();
            t.source.install(vars.as_slice()).await?;

            installed.0.insert(tid, theme);
            installed.save(&TYTM_INSTALLED)?;
        }

        Commands::List => {
            for (_, theme) in installed.0.iter() {
                println!("{}", theme.name);
            }
        }

        Commands::Remove { theme, variant } => {
            let t = installed.0.get_mut(&theme);
            let t = t.ok_or(anyhow!("Cannot find a theme with the provided ID."))?;

            if variant.is_empty() {
                t.remove_all()?;
                installed.0.remove(&theme);
            } else {
                if t.remove(
                    t.variants
                        .clone()
                        .into_iter()
                        .filter(|x| variant.contains(&x.name))
                        .collect::<Vec<_>>()
                        .as_slice(),
                )? {
                    installed.0.remove(&theme);
                }
            }

            installed.save(&TYTM_INSTALLED)?;
        }

        Commands::Update => {
            installed.update().await?;
            installed.save(&TYTM_INSTALLED)?;
        }

        Commands::Open => {
            #[cfg(target_os = "macos")]
            std::process::Command::new("open")
                .arg(TYPORA_THEME.as_os_str())
                .spawn()?;

            #[cfg(target_os = "windows")]
            std::process::Command::new("explorer")
                .arg(TYPORA_THEME.as_os_str())
                .spawn()?;

            #[cfg(target_os = "linux")]
            std::process::Command::new("xdg-open")
                .arg(TYPORA_THEME.as_os_str())
                .spawn()?;
        }
    }
    Ok(())
}
