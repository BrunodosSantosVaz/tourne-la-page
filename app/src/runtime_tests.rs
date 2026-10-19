use std::fs;

use crate::runtime::{RuntimeConfig, RuntimeError};

#[test]
fn requires_the_explicit_local_environment() {
    let root = temporary_root("wrong-environment");

    assert!(matches!(
        RuntimeConfig::new(&root, "production", None),
        Err(RuntimeError::InvalidEnvironment)
    ));
    assert!(!root.join(".local").exists());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn keeps_local_data_under_the_checkout_and_creates_only_its_local_folder() {
    let root = temporary_root("local-data");
    let config = RuntimeConfig::new(&root, "local", None).unwrap();
    let path = config.prepare().unwrap();

    assert_eq!(path, root.join(".local/tournepage").canonicalize().unwrap());
    assert!(path.is_dir());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_a_local_data_override_outside_the_checkout() {
    let root = temporary_root("outside-data");
    let outside = root.parent().unwrap().join("outside-tournepage-data");

    assert!(matches!(
        RuntimeConfig::new(&root, "local", Some(&outside)),
        Err(RuntimeError::UnsafeDataPath)
    ));
    assert!(!outside.exists());

    fs::remove_dir_all(root).unwrap();
}

fn temporary_root(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("tournepage-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn installed_builds_use_platform_data_without_requiring_a_checkout() {
    let base = temporary_root("installed-data").join("new-data-home");
    let production = RuntimeConfig::installed(&base, "production")
        .unwrap()
        .prepare()
        .unwrap();
    let staging = RuntimeConfig::installed(&base, "homologacao")
        .unwrap()
        .prepare()
        .unwrap();
    assert_eq!(
        production,
        base.join("tourne-la-page/production")
            .canonicalize()
            .unwrap()
    );
    assert_ne!(production, staging);
    assert!(RuntimeConfig::installed(&base, "unknown").is_err());
    fs::remove_dir_all(base.parent().unwrap()).unwrap();
}
