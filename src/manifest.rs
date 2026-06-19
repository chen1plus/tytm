use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io;
use std::{collections::HashMap, fs};
use std::{path::Path, sync::LazyLock};

use crate::fsx::TYPORA_THEME;
use crate::source::{Source, Variant};

pub static TYTM_REGISTRY: LazyLock<Manifest> =
    LazyLock::new(|| serde_json::from_str(include_str!("registry.json")).unwrap());

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest(pub HashMap<String, Theme>);

impl Manifest {
    pub async fn update(&mut self) -> Result<()> {
        for theme in self.0.values_mut() {
            let v = theme.version.clone().unwrap_or_default();
            theme.version = Some(theme.source.update(&theme.variants, &v).await?);
        }
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        if !path.is_file() {
            return Ok(Self(HashMap::new()));
        }
        let file = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&file)?)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        Ok(fs::write(path, serde_json::to_string(self)?)?)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Theme {
    homepage: String,
    pub name: String,
    pub source: Source,
    pub variants: Vec<Variant>,
    version: Option<String>,
}

impl Theme {
    /// Remove the variant and return true if the theme completely removed.
    pub fn remove(&mut self, variants: &[Variant]) -> io::Result<bool> {
        for variant in variants.iter() {
            if let Some(idx) = self.variants.iter().position(|x| x == variant) {
                fs::remove_file(TYPORA_THEME.join(&variant.file))?;
                self.variants.remove(idx);
            }
        }

        if self.variants.is_empty() {
            self.remove_all()?;
            return Ok(true);
        }
        Ok(false)
    }

    pub fn remove_all(&self) -> io::Result<()> {
        self.source.remove_files()?;
        for variant in self.variants.iter() {
            fs::remove_file(TYPORA_THEME.join(&variant.file))?;
        }
        Ok(())
    }
}
