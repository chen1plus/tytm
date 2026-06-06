use clap::{Parser, Subcommand};

mod cmds;
mod env;
mod fsx;
mod manifest;

#[derive(Parser)]
#[command(
    name = "TyTM",
    author = "Chen1Plus",
    about = "Typora Theme Manager",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Update the manifest
    Update,

    /// Add a new theme
    Add {
        /// The url of the theme
        url: String,
        /// The url type
        #[arg(short, long, value_name = "TYPE")]
        url_type: Option<cmds::add::UrlType>,
    },

    /// Remove a theme
    #[command(alias = "rm")]
    Remove {
        /// The theme to remove
        theme: String,

        /// The sub-packages to remove
        #[arg(short, long)]
        sub: Option<Vec<String>>,
    },

    /// List all installed themes
    #[command(alias = "ls")]
    List,
}

fn main() -> anyhow::Result<()> {
    env::init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Update => {
            todo!()
        }

        Commands::Add { url, mut url_type } => {
            if url_type.is_none() {
                if url.ends_with(".git") {
                    url_type = Some(cmds::add::UrlType::Git);
                } else if url.ends_with(".zip") {
                    url_type = Some(cmds::add::UrlType::Zip);
                } else {
                    return Err(anyhow::anyhow!("Failed to determine the url type"));
                }
            }
            // This unwrap is safe because we have already checked the url_type.
            cmds::add::entry(&url, url_type.unwrap())?;
        }

        Commands::Remove { theme, sub } => {
            cmds::remove::entry(theme, sub)?;
        }

        Commands::List => {
            cmds::list::entry()?;
        }
    }

    Ok(())
}
