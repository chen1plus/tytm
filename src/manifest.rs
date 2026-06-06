use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThemeEntry {
    pub name: String,
    pub source: String,
    pub installed_at: String,
    pub files: Vec<String>, // Relative paths to TYPORA_THEME directory
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Manifest {
    pub themes: HashMap<String, ThemeEntry>,
}

impl Manifest {
    pub fn load() -> anyhow::Result<Self> {
        let path = crate::env::manifest_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        // If manifest is empty or corrupt, fallback to default rather than erroring out
        let manifest = serde_json::from_str(&content).unwrap_or_default();
        Ok(manifest)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = crate::env::manifest_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn add_theme(&mut self, entry: ThemeEntry) {
        self.themes.insert(entry.name.clone(), entry);
    }

    pub fn remove_theme(&mut self, name: &str) -> Option<ThemeEntry> {
        self.themes.remove(name)
    }
}
