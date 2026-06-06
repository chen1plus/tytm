use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{fs, io};

#[cfg(not(debug_assertions))]
/// The user's data directory.
static DATA: LazyLock<PathBuf> =
    LazyLock::new(|| dirs::data_dir().expect("Failed to find user's data directory"));

#[cfg(debug_assertions)]
static TYPORA: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("debug").join("Typora"));
#[cfg(all(not(debug_assertions), target_os = "windows"))]
static TYPORA: LazyLock<PathBuf> = LazyLock::new(|| DATA.join("Typora"));
#[cfg(all(not(debug_assertions), target_os = "macos"))]
static TYPORA: LazyLock<PathBuf> = LazyLock::new(|| DATA.join("abnerworks.Typora"));

pub static TYPORA_THEME: LazyLock<PathBuf> = LazyLock::new(|| TYPORA.join("themes"));

#[cfg(debug_assertions)]
static TYTM: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from("debug").join("tytm"));
#[cfg(not(debug_assertions))]
static TYTM: LazyLock<PathBuf> = LazyLock::new(|| DATA.join("tytm"));

#[cfg(debug_assertions)]
pub fn init() {
    ensure_dir("debug").unwrap();
    ensure_dir(TYPORA.as_path()).unwrap();
    ensure_dir(TYPORA_THEME.as_path()).unwrap();
    ensure_dir(TYTM.as_path()).unwrap();
}

#[cfg(not(debug_assertions))]
pub fn init() {
    assert!(
        TYPORA.exists() && TYPORA.is_dir(),
        "Typora directory not found"
    );
    assert!(
        TYPORA_THEME.exists() && TYPORA_THEME.is_dir(),
        "Typora themes directory not found"
    );

    ensure_dir(TYTM.as_path()).expect("Failed to create TyTM directory");
    assert!(TYTM.is_dir());
}

pub fn manifest_path() -> PathBuf {
    TYTM.join("manifest.json")
}

/// Ensure that a directory exists, failed if missing parent directories.
#[deprecated]
fn ensure_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    if !path.as_ref().exists() {
        fs::create_dir(path)?;
    }
    Ok(())
}
