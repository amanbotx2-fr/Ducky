use std::sync::{Mutex, PoisonError};

use keyring::{Entry, Error as KeyringError};
use zeroize::{Zeroize, Zeroizing};

const KEYRING_SERVICE: &str = "com.ducky.desktop";
const MAXIMUM_SECRET_LENGTH: usize = 4_096;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CredentialId {
    AiApiKey,
}

impl CredentialId {
    const fn account(self) -> &'static str {
        match self {
            Self::AiApiKey => "ai-api-key",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CredentialMutation {
    Created,
    Updated,
    Unchanged,
    Deleted,
    Missing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CredentialStoreError {
    InvalidSecret,
    Unavailable,
}

impl std::fmt::Display for CredentialStoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSecret => formatter.write_str("credential is invalid"),
            Self::Unavailable => formatter.write_str("secure credential storage is unavailable"),
        }
    }
}

impl std::error::Error for CredentialStoreError {}

trait CredentialBackend: Send + Sync {
    fn load(&self, id: CredentialId) -> Result<Option<Zeroizing<String>>, CredentialStoreError>;
    fn save(&self, id: CredentialId, secret: &str) -> Result<(), CredentialStoreError>;
    fn delete(&self, id: CredentialId) -> Result<bool, CredentialStoreError>;
}

#[derive(Debug, Default)]
struct KeyringCredentialBackend;

impl KeyringCredentialBackend {
    fn entry(id: CredentialId) -> Result<Entry, CredentialStoreError> {
        Entry::new(KEYRING_SERVICE, id.account()).map_err(|_| CredentialStoreError::Unavailable)
    }
}

impl CredentialBackend for KeyringCredentialBackend {
    fn load(&self, id: CredentialId) -> Result<Option<Zeroizing<String>>, CredentialStoreError> {
        match Self::entry(id)?.get_password() {
            Ok(secret) => Ok(Some(Zeroizing::new(secret))),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(_) => Err(CredentialStoreError::Unavailable),
        }
    }

    fn save(&self, id: CredentialId, secret: &str) -> Result<(), CredentialStoreError> {
        Self::entry(id)?
            .set_password(secret)
            .map_err(|_| CredentialStoreError::Unavailable)
    }

    fn delete(&self, id: CredentialId) -> Result<bool, CredentialStoreError> {
        match Self::entry(id)?.delete_credential() {
            Ok(()) => Ok(true),
            Err(KeyringError::NoEntry) => Ok(false),
            Err(_) => Err(CredentialStoreError::Unavailable),
        }
    }
}

pub(crate) struct CredentialStore {
    backend: Box<dyn CredentialBackend>,
    mutation_lock: Mutex<()>,
}

impl std::fmt::Debug for CredentialStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CredentialStore")
            .finish_non_exhaustive()
    }
}

impl CredentialStore {
    pub(crate) fn native() -> Self {
        Self::with_backend(Box::<KeyringCredentialBackend>::default())
    }

    fn with_backend(backend: Box<dyn CredentialBackend>) -> Self {
        Self {
            backend,
            mutation_lock: Mutex::new(()),
        }
    }

    pub(crate) fn is_configured(&self, id: CredentialId) -> Result<bool, CredentialStoreError> {
        self.backend.load(id).map(|secret| secret.is_some())
    }

    pub(crate) fn load(
        &self,
        id: CredentialId,
    ) -> Result<Option<Zeroizing<String>>, CredentialStoreError> {
        self.backend.load(id)
    }

    pub(crate) fn save(
        &self,
        id: CredentialId,
        mut secret: String,
    ) -> Result<CredentialMutation, CredentialStoreError> {
        let normalized = secret.trim();
        if normalized.is_empty() || normalized.len() > MAXIMUM_SECRET_LENGTH {
            secret.zeroize();
            return Err(CredentialStoreError::InvalidSecret);
        }

        let _guard = self
            .mutation_lock
            .lock()
            .map_err(CredentialStoreError::from)?;
        let existing = self.backend.load(id)?;
        let mutation = match existing.as_deref() {
            Some(current) if current == normalized => CredentialMutation::Unchanged,
            Some(_) => {
                self.backend.save(id, normalized)?;
                CredentialMutation::Updated
            }
            None => {
                self.backend.save(id, normalized)?;
                CredentialMutation::Created
            }
        };
        secret.zeroize();
        Ok(mutation)
    }

    pub(crate) fn delete(
        &self,
        id: CredentialId,
    ) -> Result<CredentialMutation, CredentialStoreError> {
        let _guard = self
            .mutation_lock
            .lock()
            .map_err(CredentialStoreError::from)?;
        self.backend.delete(id).map(|deleted| {
            if deleted {
                CredentialMutation::Deleted
            } else {
                CredentialMutation::Missing
            }
        })
    }
}

impl<T> From<PoisonError<T>> for CredentialStoreError {
    fn from(_error: PoisonError<T>) -> Self {
        Self::Unavailable
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use super::*;

    #[derive(Debug, Default)]
    struct MemoryBackend {
        values: Mutex<HashMap<&'static str, String>>,
        writes: Mutex<usize>,
    }

    impl CredentialBackend for Arc<MemoryBackend> {
        fn load(
            &self,
            id: CredentialId,
        ) -> Result<Option<Zeroizing<String>>, CredentialStoreError> {
            Ok(self
                .values
                .lock()
                .map_err(CredentialStoreError::from)?
                .get(id.account())
                .cloned()
                .map(Zeroizing::new))
        }

        fn save(&self, id: CredentialId, secret: &str) -> Result<(), CredentialStoreError> {
            self.values
                .lock()
                .map_err(CredentialStoreError::from)?
                .insert(id.account(), secret.to_owned());
            *self.writes.lock().map_err(CredentialStoreError::from)? += 1;
            Ok(())
        }

        fn delete(&self, id: CredentialId) -> Result<bool, CredentialStoreError> {
            Ok(self
                .values
                .lock()
                .map_err(CredentialStoreError::from)?
                .remove(id.account())
                .is_some())
        }
    }

    fn memory_store() -> (CredentialStore, Arc<MemoryBackend>) {
        let backend = Arc::new(MemoryBackend::default());
        let store = CredentialStore::with_backend(Box::new(Arc::clone(&backend)));
        (store, backend)
    }

    #[test]
    fn credentials_save_load_update_and_delete_without_plaintext_files() {
        let (store, _) = memory_store();

        assert_eq!(store.load(CredentialId::AiApiKey).unwrap(), None);
        assert_eq!(
            store
                .save(CredentialId::AiApiKey, " first-key ".to_owned())
                .unwrap(),
            CredentialMutation::Created
        );
        let loaded = store.load(CredentialId::AiApiKey).unwrap();
        assert_eq!(
            loaded.as_deref().map(|secret| secret.as_str()),
            Some("first-key")
        );
        assert_eq!(
            store
                .save(CredentialId::AiApiKey, "replacement".to_owned())
                .unwrap(),
            CredentialMutation::Updated
        );
        assert_eq!(
            store.delete(CredentialId::AiApiKey).unwrap(),
            CredentialMutation::Deleted
        );
        assert!(!store.is_configured(CredentialId::AiApiKey).unwrap());
        assert_eq!(
            store.delete(CredentialId::AiApiKey).unwrap(),
            CredentialMutation::Missing
        );
    }

    #[test]
    fn duplicate_values_do_not_write_to_the_backend() {
        let (store, backend) = memory_store();

        store
            .save(CredentialId::AiApiKey, "same-key".to_owned())
            .unwrap();
        assert_eq!(
            store
                .save(CredentialId::AiApiKey, "same-key".to_owned())
                .unwrap(),
            CredentialMutation::Unchanged
        );
        assert_eq!(*backend.writes.lock().unwrap(), 1);
    }

    #[test]
    fn empty_and_oversized_credentials_are_rejected_without_writes() {
        let (store, backend) = memory_store();

        assert_eq!(
            store.save(CredentialId::AiApiKey, " ".to_owned()),
            Err(CredentialStoreError::InvalidSecret)
        );
        assert_eq!(
            store.save(
                CredentialId::AiApiKey,
                "x".repeat(MAXIMUM_SECRET_LENGTH + 1)
            ),
            Err(CredentialStoreError::InvalidSecret)
        );
        assert_eq!(*backend.writes.lock().unwrap(), 0);
    }
}
