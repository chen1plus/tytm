use anyhow::{Result, anyhow};
use std::{fs, io, path::Path};
use std::{path::PathBuf, sync::LazyLock};

pub static TYPORA_THEME: LazyLock<PathBuf> = LazyLock::new(|| {
    let _data = dirs::data_dir().expect("Failed to locate user's data directory.");

    #[cfg(target_os = "linux")]
    let _dir = "typora"; // TODO
    #[cfg(target_os = "macos")]
    let _dir = "abnerworks.Typora";
    #[cfg(target_os = "windows")]
    let _dir = "Typora";

    match cfg!(debug_assertions) {
        true => PathBuf::from("debug"),
        false => _data.join(_dir).join("themes"),
    }
});

pub static TYTM_INSTALLED: LazyLock<PathBuf> =
    LazyLock::new(|| TYPORA_THEME.join("tytm").join("installed.json"));

/// Find the base directory from `path` that contains the `target` file.
pub fn find_base_dir(path: &Path, target: &str) -> Result<PathBuf> {
    for file in path.read_dir()? {
        if file?
            .file_name()
            .to_str()
            .ok_or(anyhow!("Failed to convert file name to string."))?
            == target
        {
            return Ok(path.to_owned());
        }
    }

    for file in path.read_dir()? {
        let file = file?;
        if file.file_type()?.is_dir() {
            if let Ok(ret) = find_base_dir(&file.path(), target) {
                return Ok(ret);
            }
        }
    }

    Err(anyhow!("Unable to locate the base directory."))
}

/// Remove the directory or file at `path`.
pub fn remove_item(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}
