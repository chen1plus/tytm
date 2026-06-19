use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{fs, io};

use crate::{fsx, fsx::TYPORA_THEME};

/// Various theme sources, sufficient for install, remove and update operations.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Source {
    GhReleaseZip(GhReleaseZipInner),
    Git(GitInner),
    Zip(ZipInner),
}

/// A variant represents the smallest unit of a theme that can be applied in Typora.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Variant {
    pub file: String,
    name: String,
}

impl Source {
    /// Install the theme and return the version.
    pub async fn install(&self, variants: &[Variant]) -> Result<String> {
        match &self {
            Self::GhReleaseZip(x) => x.install(variants).await,
            Self::Git(x) => x.install(variants).await,
            Self::Zip(x) => x.install(variants).await,
        }
    }

    /// Update the theme and return the new version.
    pub async fn update(&self, variants: &[Variant], version: &str) -> Result<String> {
        if match self {
            Source::GhReleaseZip(x) => &x.gh_latest().await?.tag_name == version,
            Source::Git(x) => !x.outdated(version)?,
            Source::Zip(_) => false,
        } {
            return Ok(version.to_string());
        }

        self.remove_files()?; // Update the themes.
        self.install(variants).await
    }

    /// Remove the assets.
    pub fn remove_files(&self) -> io::Result<()> {
        for f in match &self {
            Self::GhReleaseZip(x) => x.files.iter(),
            Self::Git(x) => x.files.iter(),
            Self::Zip(x) => x.files.iter(),
        } {
            fsx::remove_item(&TYPORA_THEME.join(f))?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GhReleaseZipInner {
    files: Vec<String>,
    gh_owner: String,
    gh_repo: String,
    regex: String,
}

impl GhReleaseZipInner {
    /// Get the latest release from GitHub.
    async fn gh_latest(&self) -> octocrab::Result<octocrab::models::repos::Release> {
        octocrab::instance()
            .repos(&self.gh_owner, &self.gh_repo)
            .releases()
            .get_latest()
            .await
    }

    async fn install(&self, variants: &[Variant]) -> Result<String> {
        let gh_latest = self.gh_latest().await?;
        let re = regex::Regex::new(&self.regex)?;
        let asset = gh_latest
            .assets
            .iter()
            .find(|x| re.is_match(&x.name))
            .ok_or(anyhow!("No assets matched the pattern."))?;

        // Reuse the zip installation logic.
        ZipInner {
            files: self.files.clone(),
            url: asset.browser_download_url.to_string(),
        }
        .install(variants)
        .await?;

        // Return the version of the installed theme.
        Ok(gh_latest.tag_name)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GitInner {
    files: Vec<String>,
    url: String,
}

impl GitInner {
    async fn install(&self, variants: &[Variant]) -> Result<String> {
        let tmp_dir = tempfile::tempdir()?;
        let repo = git2::Repository::clone(&self.url, &tmp_dir)?;
        let commit = repo.head()?.peel_to_commit()?.id().to_string();

        // Find the base directory that contains the theme files.
        let v = variants.first().expect("At least one variant is required.");
        let base = fsx::find_base_dir(tmp_dir.path(), &v.file)?;

        // Copy files, including assets and variants.
        for file in self.files.iter().chain(variants.iter().map(|x| &x.file)) {
            fs::rename(base.join(file), TYPORA_THEME.join(file))?;
        }

        // Return the commit hash as the version.
        Ok(commit)
    }

    fn outdated(&self, commit: &str) -> Result<bool> {
        let mut repo = git2::Remote::create_detached(self.url.clone())?;
        repo.connect(git2::Direction::Fetch)?;

        // Get the latest commit hash.
        let hash = repo
            .list()?
            .iter()
            .find(|x| x.name() == "HEAD")
            .ok_or(anyhow!("Could not find the default branch."))?
            .oid();

        // Compare the latest commit hash with the provided one.
        Ok(commit != hash.to_string())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ZipInner {
    files: Vec<String>,
    url: String,
}

impl ZipInner {
    async fn install(&self, variants: &[Variant]) -> Result<String> {
        let tmp_dir = tempfile::tempdir()?;

        // Download the zip file and extract it to a temporary directory.
        let mut tmp_file = tempfile::tempfile()?;
        let response = reqwest::get(&self.url).await?;
        io::copy(&mut &response.bytes().await?[..], &mut tmp_file)?;
        zip::ZipArchive::new(tmp_file)?.extract(&tmp_dir)?;

        // Find the base directory that contains the theme files.
        let v = variants.first().expect("At least one variant is required.");
        let base = fsx::find_base_dir(tmp_dir.path(), &v.file)?;

        // Copy files, including assets and variants.
        for file in self.files.iter().chain(variants.iter().map(|x| &x.file)) {
            fs::rename(base.join(file), TYPORA_THEME.join(file))?;
        }

        // Return the time of installation as the version.
        Ok(chrono::Utc::now().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn gh_release_zip_install() {
        let source = r#"{
            "files": ["lapis"],
            "gh_owner": "YiNNx",
            "gh_repo": "typora-theme-lapis",
            "regex": "typora-(.+)",
            "type": "GhReleaseZip"
        }"#;

        let variants = r#"[
            { "file": "lapis-dark.css", "name": "Dark" },
            { "file": "lapis.css", "name": "Light" }
        ]"#;

        let s: Source = serde_json::from_str(source).unwrap();
        let v: Vec<Variant> = serde_json::from_str(variants).unwrap();
        assert!(s.install(v.as_slice()).await.unwrap() == "v1.2.1");
    }

    async fn git_install() {
        let source = r#"{
            "files": ["maize"],
            "type": "Git",
            "url": "https://github.com/BEATREE/typora-maize-theme"
        }"#;

        let variants = r#"[
            { "file": "maize.css", "name": "Maize" }
        ]"#;

        let s: Source = serde_json::from_str(source).unwrap();
        let v: Vec<Variant> = serde_json::from_str(variants).unwrap();
        let version = s.install(v.as_slice()).await.unwrap();
        assert!(version == "89186ead834f90b8df46ee1d12aafe1de431fdd4");
    }

    #[tokio::test]
    async fn install() {
        fs::remove_dir_all(TYPORA_THEME.as_path()).unwrap_or_default();
        fs::create_dir_all(TYPORA_THEME.join("tytm")).unwrap();
        gh_release_zip_install().await;
        git_install().await;
    }
}
