use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;

use crate::domain::pomodoro::{
    PersistedPomodoroDocument, PomodoroLoad, PomodoroRepository, PomodoroRepositoryError,
};

#[derive(Clone, Debug)]
pub(crate) struct PomodoroStore {
    path: PathBuf,
}

impl PomodoroStore {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    fn save_document(
        &self,
        document: &PersistedPomodoroDocument,
    ) -> Result<(), PomodoroRepositoryError> {
        document
            .validate()
            .map_err(|error| PomodoroRepositoryError::new(error.to_string()))?;
        let serialized = format!(
            "{}\n",
            serde_json::to_string_pretty(document)
                .map_err(|error| repository_error("Pomodoro JSON failed", error))?
        );
        let directory = self.path.parent().ok_or_else(|| {
            PomodoroRepositoryError::new("Pomodoro path has no parent directory.")
        })?;

        fs::create_dir_all(directory)
            .map_err(|error| repository_error("Pomodoro directory creation failed", error))?;
        let mut temporary = NamedTempFile::new_in(directory)
            .map_err(|error| repository_error("Pomodoro temporary file failed", error))?;
        set_private_permissions(temporary.as_file())
            .map_err(|error| repository_error("Pomodoro permissions failed", error))?;
        temporary
            .write_all(serialized.as_bytes())
            .map_err(|error| repository_error("Pomodoro write failed", error))?;
        temporary
            .as_file()
            .sync_all()
            .map_err(|error| repository_error("Pomodoro file sync failed", error))?;
        temporary
            .persist(&self.path)
            .map_err(|error| repository_error("Pomodoro atomic replace failed", error.error))?;
        sync_directory(directory)
            .map_err(|error| repository_error("Pomodoro directory sync failed", error))?;
        Ok(())
    }
}

impl PomodoroRepository for PomodoroStore {
    fn load(&self) -> Result<PomodoroLoad, PomodoroRepositoryError> {
        let serialized = match fs::read_to_string(&self.path) {
            Ok(serialized) => serialized,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(PomodoroLoad::Missing);
            }
            Err(error) => {
                return Err(repository_error("Pomodoro read failed", error));
            }
        };

        Ok(parse_document(&serialized)
            .map(PomodoroLoad::Document)
            .unwrap_or(PomodoroLoad::Invalid))
    }

    fn save(&self, document: &PersistedPomodoroDocument) -> Result<(), PomodoroRepositoryError> {
        self.save_document(document)
    }
}

fn parse_document(serialized: &str) -> Option<PersistedPomodoroDocument> {
    let document: PersistedPomodoroDocument = serde_json::from_str(serialized).ok()?;
    document.validate().ok()?;
    Some(document)
}

fn repository_error(context: &str, error: impl std::fmt::Display) -> PomodoroRepositoryError {
    PomodoroRepositoryError::new(format!("{context}: {error}"))
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
    use super::*;
    use crate::domain::pomodoro::{PomodoroState, DOCUMENT_VERSION};
    use tempfile::tempdir;

    fn store_in(directory: &Path) -> PomodoroStore {
        PomodoroStore::new(directory.join("pomodoro.json"))
    }

    #[test]
    fn missing_store_does_not_materialize_by_itself() {
        let directory = tempdir().expect("temporary directory");
        let store = store_in(directory.path());

        assert_eq!(store.load().expect("missing load"), PomodoroLoad::Missing);
        assert!(!store.path().exists());
    }

    #[test]
    fn document_round_trips_with_the_exact_versioned_schema() {
        let directory = tempdir().expect("temporary directory");
        let store = store_in(directory.path());
        let document = PersistedPomodoroDocument::new(PomodoroState {
            running: true,
            paused: true,
            selected_duration_minutes: 50,
            duration_minutes: 50,
            remaining_seconds: 2_400,
            started_at: Some(1_000),
        });

        store.save(&document).expect("save");
        assert_eq!(
            store.load().expect("reload"),
            PomodoroLoad::Document(document)
        );
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(store.path()).expect("stored JSON"))
                .expect("JSON value");
        assert_eq!(value["version"], DOCUMENT_VERSION);
        assert_eq!(value["state"]["selectedDurationMinutes"], 50);
        assert_eq!(value["state"]["remainingSeconds"], 2_400);
    }

    #[test]
    fn invalid_file_remains_available_and_loads_as_invalid() {
        let directory = tempdir().expect("temporary directory");
        let store = store_in(directory.path());
        fs::write(store.path(), "{ invalid json").expect("invalid write");

        assert_eq!(store.load().expect("invalid load"), PomodoroLoad::Invalid);
        assert_eq!(
            fs::read_to_string(store.path()).expect("invalid retained"),
            "{ invalid json"
        );
    }

    #[cfg(unix)]
    #[test]
    fn saved_file_is_owner_only() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempdir().expect("temporary directory");
        let store = store_in(directory.path());
        store
            .save(&PersistedPomodoroDocument::new(PomodoroState::default()))
            .expect("save");
        let mode = fs::metadata(store.path())
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777;

        assert_eq!(mode, 0o600);
    }
}
