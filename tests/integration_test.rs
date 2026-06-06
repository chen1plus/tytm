use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

fn get_binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tytm"))
}

fn create_dummy_theme_repo(dir: &Path) -> PathBuf {
    let repo_dir = dir.join("dummy-theme-repo.git");
    fs::create_dir_all(&repo_dir).unwrap();

    // Init git repo
    let repo = git2::Repository::init(&repo_dir).unwrap();

    // Create dummy theme files
    let css_file = repo_dir.join("dummy-theme.css");
    fs::write(&css_file, "body { background: #fff; }").unwrap();

    let theme_sub_dir = repo_dir.join("dummy-theme");
    fs::create_dir_all(&theme_sub_dir).unwrap();
    fs::write(theme_sub_dir.join("some-asset.txt"), "asset content").unwrap();

    // Add files to index
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("dummy-theme.css")).unwrap();
    index.add_path(Path::new("dummy-theme/some-asset.txt")).unwrap();
    index.write().unwrap();

    // Commit changes
    let oid = index.write_tree().unwrap();
    let tree = repo.find_tree(oid).unwrap();
    let signature = git2::Signature::now("Test User", "test@example.com").unwrap();
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit of dummy theme",
        &tree,
        &[],
    )
    .unwrap();

    repo_dir
}

#[test]
fn test_empty_list() {
    let temp = tempdir().unwrap();
    let bin = get_binary_path();

    let output = Command::new(&bin)
        .arg("ls")
        .current_dir(temp.path())
        .output()
        .expect("Failed to run tytm ls");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No themes found in Typora themes directory."));
}

#[test]
fn test_add_list_and_remove() {
    let temp = tempdir().unwrap();
    let bin = get_binary_path();

    // Create the dummy git repository
    let repo_path = create_dummy_theme_repo(temp.path());

    // 1. Add theme
    let output = Command::new(&bin)
        .arg("add")
        .arg(repo_path.to_str().unwrap())
        .current_dir(temp.path())
        .output()
        .expect("Failed to run tytm add");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Theme 'dummy-theme' successfully installed!"));

    // Verify files got copied into debug directory
    let themes_dir = temp.path().join("debug").join("Typora").join("themes");
    assert!(themes_dir.join("dummy-theme.css").exists());
    assert!(themes_dir.join("dummy-theme").join("some-asset.txt").exists());

    // Verify manifest was created
    let manifest_path = temp.path().join("debug").join("tytm").join("manifest.json");
    assert!(manifest_path.exists());
    let manifest_content = fs::read_to_string(manifest_path).unwrap();
    assert!(manifest_content.contains("dummy-theme"));

    // 2. List themes
    let output = Command::new(&bin)
        .arg("ls")
        .current_dir(temp.path())
        .output()
        .expect("Failed to run tytm ls");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Managed"));
    assert!(stdout.contains("dummy-theme"));

    // 3. Remove theme
    let output = Command::new(&bin)
        .arg("rm")
        .arg("dummy-theme")
        .current_dir(temp.path())
        .output()
        .expect("Failed to run tytm rm");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Theme 'dummy-theme' removed successfully."));

    // Verify files got deleted
    assert!(!themes_dir.join("dummy-theme.css").exists());
    assert!(!themes_dir.join("dummy-theme").exists());

    // Verify manifest no longer contains theme
    let manifest_path = temp.path().join("debug").join("tytm").join("manifest.json");
    let manifest_content = fs::read_to_string(manifest_path).unwrap();
    assert!(!manifest_content.contains("dummy-theme"));
}

#[test]
fn test_manifest_version_mismatch_rebuild() {
    let temp = tempdir().unwrap();
    let bin = get_binary_path();

    // 1. Create a manifest file with a future/mismatched version "2"
    let manifest_dir = temp.path().join("debug").join("tytm");
    fs::create_dir_all(&manifest_dir).unwrap();
    let manifest_path = manifest_dir.join("manifest.json");
    
    let bad_manifest = r#"{
      "manifest_version": "2",
      "themes": {
        "legacy-theme": {
          "name": "legacy-theme",
          "source": "some-source",
          "version": "v2.0",
          "installed_at": "2026-06-07 00:00:00",
          "files": []
        }
      }
    }"#;
    fs::write(&manifest_path, bad_manifest).unwrap();

    // 2. Run list command
    let output = Command::new(&bin)
        .arg("ls")
        .current_dir(temp.path())
        .output()
        .expect("Failed to run tytm ls");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Since the manifest version was mismatched, it should rebuild it as empty,
    // so no themes should be found
    assert!(stdout.contains("No themes found in Typora themes directory."));
    
    // 3. Now let's add a dummy theme to trigger a save, and verify it wrote version "1"
    let repo_path = create_dummy_theme_repo(temp.path());
    let output = Command::new(&bin)
        .arg("add")
        .arg(repo_path.to_str().unwrap())
        .current_dir(temp.path())
        .output()
        .expect("Failed to run tytm add");

    assert!(output.status.success());
    
    // Read the manifest again and verify it is version "1" and legacy-theme is GONE (rebuilt)
    let manifest_content = fs::read_to_string(&manifest_path).unwrap();
    assert!(manifest_content.contains(r#""manifest_version": "1""#));
    assert!(manifest_content.contains("dummy-theme"));
    assert!(!manifest_content.contains("legacy-theme"));
}
