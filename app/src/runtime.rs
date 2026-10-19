use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("TOURNEPAGE_ENV must be local, homologacao or production")]
    InvalidEnvironment,
    #[error("the operating system did not provide an application data directory")]
    MissingDataDirectory,
    #[error("local data must stay inside .local/tournepage in this checkout")]
    UnsafeDataPath,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub struct RuntimeConfig {
    root: PathBuf,
    data_dir: PathBuf,
}

impl RuntimeConfig {
    pub fn from_environment(root: &Path) -> Result<Self, RuntimeError> {
        let environment = env::var("TOURNEPAGE_ENV").unwrap_or_else(|_| "production".into());
        if environment != "local" {
            return Self::installed(&installed_data_home()?, &environment);
        }
        let override_path = env::var_os("TOURNEPAGE_DATA_DIR").map(PathBuf::from);
        Self::new(root, &environment, override_path.as_deref())
    }

    pub fn installed(data_home: &Path, environment: &str) -> Result<Self, RuntimeError> {
        if !matches!(environment, "production" | "homologacao") {
            return Err(RuntimeError::InvalidEnvironment);
        }
        if !data_home.is_absolute() {
            return Err(RuntimeError::UnsafeDataPath);
        }
        let root = normalize_path(data_home);
        let data_dir = root.join("tourne-la-page").join(environment);
        Ok(Self { root, data_dir })
    }

    pub fn new(
        root: &Path,
        environment: &str,
        override_path: Option<&Path>,
    ) -> Result<Self, RuntimeError> {
        if environment != "local" {
            return Err(RuntimeError::InvalidEnvironment);
        }
        let root = fs::canonicalize(root)?;
        let allowed = root.join(".local/tournepage");
        let requested =
            override_path.map_or_else(|| allowed.clone(), |path| absolute_path(&root, path));
        let data_dir = normalize_path(&requested);
        if !data_dir.starts_with(&allowed) {
            return Err(RuntimeError::UnsafeDataPath);
        }
        Ok(Self { root, data_dir })
    }

    pub fn prepare(&self) -> Result<PathBuf, RuntimeError> {
        fs::create_dir_all(&self.root)?;
        let root = fs::canonicalize(&self.root)?;
        reject_symlinks(&self.root, &self.data_dir)?;
        fs::create_dir_all(&self.data_dir)?;
        let data_dir = fs::canonicalize(&self.data_dir)?;
        if !data_dir.starts_with(&root) {
            return Err(RuntimeError::UnsafeDataPath);
        }
        Ok(data_dir)
    }
}

fn installed_data_home() -> Result<PathBuf, RuntimeError> {
    #[cfg(target_os = "windows")]
    let home = env::var_os("LOCALAPPDATA")
        .or_else(|| env::var_os("APPDATA"))
        .map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let home = env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let home = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")));
    home.ok_or(RuntimeError::MissingDataDirectory)
}

fn absolute_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_owned()
    } else {
        root.join(path)
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn reject_symlinks(root: &Path, target: &Path) -> Result<(), RuntimeError> {
    let relative = target
        .strip_prefix(root)
        .map_err(|_| RuntimeError::UnsafeDataPath)?;
    let mut current = root.to_owned();
    for component in relative.components() {
        current.push(component.as_os_str());
        if fs::symlink_metadata(&current).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
            return Err(RuntimeError::UnsafeDataPath);
        }
    }
    Ok(())
}
