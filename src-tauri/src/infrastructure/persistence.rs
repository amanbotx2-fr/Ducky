use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use tempfile::NamedTempFile;

use crate::domain::settings::{SettingsDocument, SettingsValidationError};

#[derive(Debug)]
pub(crate) enum SettingsStoreError {
    Io(io::Error),
    Json(serde_json::Error),
    Validation(SettingsValidationError),
}

impl std::fmt::Display for SettingsStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "settings I/O failed: {error}"),
            Self::Json(error) => write!(formatter, "settings JSON failed: {error}"),
            Self::Validation(error) => write!(formatter, "settings validation failed: {error}"),
        }
    }
}

impl std::error::Error for SettingsStoreError {}

impl From<io::Error> for SettingsStoreError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for SettingsStoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<SettingsValidationError> for SettingsStoreError {
    fn from(error: SettingsValidationError) -> Self {
        Self::Validation(error)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn load(&self) -> Result<SettingsDocument, SettingsStoreError> {
        match fs::read_to_string(&self.path) {
            Ok(serialized) => match self.parse(&serialized) {
                Ok(settings) => Ok(settings),
                Err(_) => self.recover_invalid_file(),
            },
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let settings = SettingsDocument::default();
                self.save(&settings)?;
                Ok(settings)
            }
            Err(error) => Err(error.into()),
        }
    }

    pub(crate) fn load_with_legacy(
        &self,
        legacy_path: Option<&Path>,
    ) -> Result<SettingsDocument, SettingsStoreError> {
        if !self.path.exists() {
            if let Some(legacy_path) = legacy_path.filter(|path| path.exists()) {
                match fs::read_to_string(legacy_path)
                    .map_err(SettingsStoreError::from)
                    .and_then(|serialized| self.parse(&serialized))
                {
                    Ok(settings) => {
                        self.save(&settings)?;
                        eprintln!(
                            "[settings] imported Electron settings from {}",
                            legacy_path.display()
                        );
                        return Ok(settings);
                    }
                    Err(error) => {
                        eprintln!("[settings] Electron settings import skipped: {error}");
                    }
                }
            }
        }

        self.load()
    }

    pub(crate) fn save(&self, settings: &SettingsDocument) -> Result<(), SettingsStoreError> {
        settings.validate()?;
        let serialized = format!("{}\n", serde_json::to_string_pretty(settings)?);
        let directory = self.path.parent().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "settings path has no parent")
        })?;

        fs::create_dir_all(directory)?;
        let mut temporary = NamedTempFile::new_in(directory)?;
        set_private_permissions(temporary.as_file())?;
        temporary.write_all(serialized.as_bytes())?;
        temporary.as_file().sync_all()?;
        temporary
            .persist(&self.path)
            .map_err(|error| SettingsStoreError::Io(error.error))?;
        sync_directory(directory)?;
        Ok(())
    }

    fn parse(&self, serialized: &str) -> Result<SettingsDocument, SettingsStoreError> {
        let value = serde_json::from_str(serialized)?;
        Ok(SettingsDocument::parse(value)?)
    }

    fn recover_invalid_file(&self) -> Result<SettingsDocument, SettingsStoreError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let recovery_path = PathBuf::from(format!(
            "{}.invalid-{timestamp}",
            self.path.to_string_lossy()
        ));

        match fs::rename(&self.path, recovery_path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        let settings = SettingsDocument::default();
        self.save(&settings)?;
        Ok(settings)
    }
}

#[cfg(unix)]
fn set_private_permissions(file: &fs::File) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    file.set_permissions(fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_private_permissions(_file: &fs::File) -> io::Result<()> {
    Ok(())
}

#[cfg(unix)]
fn sync_directory(directory: &Path) -> io::Result<()> {
    fs::File::open(directory)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_directory: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::json;
    use tempfile::tempdir;

    use super::*;

    fn store_in(directory: &Path) -> SettingsStore {
        SettingsStore::new(directory.join("settings.json"))
    }

    #[test]
    fn missing_store_materializes_and_restores_defaults() {
        let directory = tempdir().expect("temporary directory");
        let store = store_in(directory.path());

        let settings = store.load().expect("defaults load");
        assert_eq!(settings, SettingsDocument::default());
        assert_eq!(
            store.load().expect("persisted defaults reload"),
            SettingsDocument::default()
        );
    }

    #[test]
    fn saved_settings_round_trip_without_losing_deferred_data() {
        let directory = tempdir().expect("temporary directory");
        let store = store_in(directory.path());
        let mut settings = SettingsDocument::default();
        settings.general.always_on_top = false;
        settings.notification_sounds.volume = 25;
        settings.reminders.push(json!({ "id": "phase-seven" }));
        settings.credential = Some(json!({
            "version": 1,
            "ciphertext": "dGVzdA=="
        }));

        store.save(&settings).expect("settings save");
        assert_eq!(store.load().expect("settings reload"), settings);
        assert!(!directory.path().join("settings.json.tmp").exists());
    }

    #[test]
    fn invalid_store_is_recovered_and_kept_for_diagnostics() {
        let directory = tempdir().expect("temporary directory");
        let store = store_in(directory.path());
        fs::write(store.path(), "{ invalid json").expect("invalid fixture write");

        assert_eq!(
            store.load().expect("recovered defaults"),
            SettingsDocument::default()
        );

        let recovered_files = fs::read_dir(directory.path())
            .expect("settings directory")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("settings.json.invalid-")
            })
            .count();
        assert_eq!(recovered_files, 1);
    }

    #[cfg(unix)]
    #[test]
    fn settings_file_permissions_are_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempdir().expect("temporary directory");
        let store = store_in(directory.path());
        store
            .save(&SettingsDocument::default())
            .expect("settings save");

        let mode = fs::metadata(store.path())
            .expect("settings metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn legacy_settings_are_imported_once_without_mutating_the_source() {
        let directory = tempdir().expect("temporary directory");
        let native = directory.path().join("native").join("settings.json");
        let legacy = directory.path().join("electron").join("settings.json");
        fs::create_dir_all(legacy.parent().expect("legacy directory"))
            .expect("legacy directory create");
        let mut legacy_settings = SettingsDocument::default();
        legacy_settings.user_name = "Legacy Friend".to_owned();
        let serialized = format!(
            "{}\n",
            serde_json::to_string_pretty(&legacy_settings).unwrap()
        );
        fs::write(&legacy, &serialized).expect("legacy settings write");
        let store = SettingsStore::new(native);

        let imported = store
            .load_with_legacy(Some(&legacy))
            .expect("legacy settings import");
        assert_eq!(imported.user_name, "Legacy Friend");
        assert_eq!(fs::read_to_string(&legacy).unwrap(), serialized);

        let mut native_settings = imported;
        native_settings.user_name = "Native Friend".to_owned();
        store
            .save(&native_settings)
            .expect("native settings update");
        assert_eq!(
            store
                .load_with_legacy(Some(&legacy))
                .expect("native settings reload")
                .user_name,
            "Native Friend"
        );
    }
}
