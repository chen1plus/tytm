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

    /// Update all installed themes
    Update,
}

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

        Commands::Update => {
            installed.update().await?;
            installed.save(&TYTM_INSTALLED)?;
        }
    }
    Ok(())
}
