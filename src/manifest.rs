use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThemeEntry {
    pub name: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub installed_at: String,
    pub files: Vec<String>, // Relative paths to TYPORA_THEME directory
}

fn default_manifest_version() -> String {
    "1".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    #[serde(default = "default_manifest_version")]
    pub manifest_version: String,
    pub themes: HashMap<String, ThemeEntry>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            manifest_version: default_manifest_version(),
            themes: HashMap::new(),
        }
    }
}

impl Manifest {
    pub fn load() -> anyhow::Result<Self> {
        let path = crate::env::manifest_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        // If manifest is empty or corrupt, fallback to default rather than erroring out
        let manifest: Self = serde_json::from_str(&content).unwrap_or_default();
        if manifest.manifest_version != default_manifest_version() {
            return Ok(Self::default());
        }
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
