use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::Notify;

const MAXIMUM_VERSION_LENGTH: usize = 64;
const MAXIMUM_ERROR_CODE_LENGTH: usize = 80;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "phase", rename_all = "kebab-case")]
pub(crate) enum UpdateStatus {
    #[serde(rename_all = "camelCase")]
    Idle { current_version: String },
    #[serde(rename_all = "camelCase")]
    Checking { current_version: String },
    #[serde(rename_all = "camelCase")]
    Available {
        current_version: String,
        available_version: String,
    },
    #[serde(rename = "not-available", rename_all = "camelCase")]
    NotAvailable { current_version: String },
    #[serde(rename_all = "camelCase")]
    Downloading {
        current_version: String,
        available_version: Option<String>,
        percent: f64,
        transferred_bytes: f64,
        total_bytes: f64,
        bytes_per_second: f64,
    },
    #[serde(rename_all = "camelCase")]
    Downloaded {
        current_version: String,
        available_version: String,
    },
    #[serde(rename_all = "camelCase")]
    Error {
        current_version: String,
        message: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UpdateRelease {
    pub(crate) version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UpdateBackendError {
    code: String,
}

impl UpdateBackendError {
    pub(crate) fn new(code: impl AsRef<str>) -> Self {
        let normalized = code.as_ref().trim();
        let code = if normalized.is_empty() {
            "unknown".to_owned()
        } else {
            normalized.chars().take(MAXIMUM_ERROR_CODE_LENGTH).collect()
        };

        Self { code }
    }

    pub(crate) fn code(&self) -> &str {
        &self.code
    }
}

#[async_trait]
pub(crate) trait UpdateBackend: Send + Sync {
    async fn check(&self) -> Result<Option<UpdateRelease>, UpdateBackendError>;
}

type UpdateStatusSink = Arc<dyn Fn(UpdateStatus) + Send + Sync>;

struct RuntimeState {
    status: UpdateStatus,
    automatic_checks_enabled: bool,
    active_check: bool,
    check_generation: u64,
}

#[derive(Clone)]
pub(crate) struct UpdaterRuntime {
    current_version: String,
    is_packaged: bool,
    backend: Arc<dyn UpdateBackend>,
    status_sink: UpdateStatusSink,
    state: Arc<Mutex<RuntimeState>>,
    check_completed: Arc<Notify>,
}

impl UpdaterRuntime {
    pub(crate) fn new(
        current_version: impl AsRef<str>,
        is_packaged: bool,
        backend: Arc<dyn UpdateBackend>,
        status_sink: UpdateStatusSink,
    ) -> Self {
        let current_version = normalize_version(current_version.as_ref());
        Self {
            state: Arc::new(Mutex::new(RuntimeState {
                status: UpdateStatus::Idle {
                    current_version: current_version.clone(),
                },
                automatic_checks_enabled: false,
                active_check: false,
                check_generation: 0,
            })),
            current_version,
            is_packaged,
            backend,
            status_sink,
            check_completed: Arc::new(Notify::new()),
        }
    }

    pub(crate) fn get_status(&self) -> Result<UpdateStatus, UpdaterRuntimeError> {
        self.state
            .lock()
            .map(|state| state.status.clone())
            .map_err(|_| UpdaterRuntimeError::StateUnavailable)
    }

    pub(crate) fn set_automatic_checks_enabled(
        &self,
        enabled: bool,
    ) -> Result<bool, UpdaterRuntimeError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| UpdaterRuntimeError::StateUnavailable)?;
        let became_enabled = enabled && !state.automatic_checks_enabled;
        state.automatic_checks_enabled = enabled;
        Ok(became_enabled)
    }

    pub(crate) async fn check_automatically(&self) -> Result<UpdateStatus, UpdaterRuntimeError> {
        let enabled = self
            .state
            .lock()
            .map_err(|_| UpdaterRuntimeError::StateUnavailable)?
            .automatic_checks_enabled;

        if !enabled {
            return self.get_status();
        }

        self.start_check(CheckSource::Automatic).await
    }

    pub(crate) async fn check_for_updates(&self) -> Result<UpdateStatus, UpdaterRuntimeError> {
        self.start_check(CheckSource::Manual).await
    }

    async fn start_check(&self, source: CheckSource) -> Result<UpdateStatus, UpdaterRuntimeError> {
        if !self.is_packaged {
            return match source {
                CheckSource::Automatic => self.get_status(),
                CheckSource::Manual => self.set_status(UpdateStatus::Error {
                    current_version: self.current_version.clone(),
                    message: "Update checks are available in packaged builds.".to_owned(),
                }),
            };
        }

        let check_role = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| UpdaterRuntimeError::StateUnavailable)?;
            if state.active_check {
                CheckRole::Follower(state.check_generation)
            } else {
                state.active_check = true;
                state.status = UpdateStatus::Checking {
                    current_version: self.current_version.clone(),
                };
                CheckRole::Leader(state.status.clone())
            }
        };

        match check_role {
            CheckRole::Leader(status) => {
                (self.status_sink)(status);
                self.run_check().await
            }
            CheckRole::Follower(check_generation) => self.wait_for_check(check_generation).await,
        }
    }

    async fn run_check(&self) -> Result<UpdateStatus, UpdaterRuntimeError> {
        let status = match self.backend.check().await {
            Ok(Some(release)) => UpdateStatus::Available {
                current_version: self.current_version.clone(),
                available_version: normalize_version(&release.version),
            },
            Ok(None) => UpdateStatus::NotAvailable {
                current_version: self.current_version.clone(),
            },
            Err(error) => {
                eprintln!("[updates] check_failed: errorCode={}", error.code());
                UpdateStatus::Error {
                    current_version: self.current_version.clone(),
                    message: "Unable to check for updates.".to_owned(),
                }
            }
        };

        let result = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| UpdaterRuntimeError::StateUnavailable)?;
            state.status = status.clone();
            state.active_check = false;
            state.check_generation = state.check_generation.wrapping_add(1);
            state.status.clone()
        };

        (self.status_sink)(result.clone());
        self.check_completed.notify_waiters();
        Ok(result)
    }

    async fn wait_for_check(
        &self,
        check_generation: u64,
    ) -> Result<UpdateStatus, UpdaterRuntimeError> {
        loop {
            let notified = self.check_completed.notified();
            {
                let state = self
                    .state
                    .lock()
                    .map_err(|_| UpdaterRuntimeError::StateUnavailable)?;
                if !state.active_check || state.check_generation != check_generation {
                    return Ok(state.status.clone());
                }
            }
            notified.await;
        }
    }

    fn set_status(&self, status: UpdateStatus) -> Result<UpdateStatus, UpdaterRuntimeError> {
        let result = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| UpdaterRuntimeError::StateUnavailable)?;
            state.status = status;
            state.status.clone()
        };
        (self.status_sink)(result.clone());
        Ok(result)
    }
}

#[derive(Clone, Copy)]
enum CheckSource {
    Automatic,
    Manual,
}

enum CheckRole {
    Leader(UpdateStatus),
    Follower(u64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UpdaterRuntimeError {
    StateUnavailable,
}

impl std::fmt::Display for UpdaterRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("updater state is unavailable")
    }
}

impl std::error::Error for UpdaterRuntimeError {}

fn normalize_version(value: &str) -> String {
    let version = value.trim();
    if version.is_empty() || version.chars().count() > MAXIMUM_VERSION_LENGTH {
        "Unknown".to_owned()
    } else {
        version.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use tokio::sync::Notify;

    use super::*;

    struct StaticBackend {
        result: Result<Option<UpdateRelease>, UpdateBackendError>,
        calls: AtomicUsize,
    }

    impl StaticBackend {
        fn new(result: Result<Option<UpdateRelease>, UpdateBackendError>) -> Self {
            Self {
                result,
                calls: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl UpdateBackend for StaticBackend {
        async fn check(&self) -> Result<Option<UpdateRelease>, UpdateBackendError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.result.clone()
        }
    }

    struct BlockingBackend {
        calls: AtomicUsize,
        started: Notify,
        released: Notify,
    }

    #[async_trait]
    impl UpdateBackend for BlockingBackend {
        async fn check(&self) -> Result<Option<UpdateRelease>, UpdateBackendError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            self.started.notify_waiters();
            self.released.notified().await;
            Ok(None)
        }
    }

    fn runtime(
        backend: Arc<dyn UpdateBackend>,
        is_packaged: bool,
    ) -> (UpdaterRuntime, Arc<Mutex<Vec<UpdateStatus>>>) {
        let statuses = Arc::new(Mutex::new(Vec::new()));
        let sink_statuses = Arc::clone(&statuses);
        (
            UpdaterRuntime::new(
                "1.2.3",
                is_packaged,
                backend,
                Arc::new(move |status| {
                    sink_statuses.lock().unwrap().push(status);
                }),
            ),
            statuses,
        )
    }

    #[tokio::test]
    async fn automatic_checks_remain_disabled_until_enabled() {
        let backend = Arc::new(StaticBackend::new(Ok(None)));
        let (runtime, statuses) = runtime(backend.clone(), true);

        assert_eq!(
            runtime.check_automatically().await.unwrap(),
            UpdateStatus::Idle {
                current_version: "1.2.3".to_owned()
            }
        );
        assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
        assert!(statuses.lock().unwrap().is_empty());

        assert!(runtime.set_automatic_checks_enabled(true).unwrap());
        assert_eq!(
            runtime.check_automatically().await.unwrap(),
            UpdateStatus::NotAvailable {
                current_version: "1.2.3".to_owned()
            }
        );
        assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn manual_checks_report_available_versions_and_status_events() {
        let backend = Arc::new(StaticBackend::new(Ok(Some(UpdateRelease {
            version: " 2.0.0 ".to_owned(),
        }))));
        let (runtime, statuses) = runtime(backend, true);

        assert_eq!(
            runtime.check_for_updates().await.unwrap(),
            UpdateStatus::Available {
                current_version: "1.2.3".to_owned(),
                available_version: "2.0.0".to_owned(),
            }
        );
        assert_eq!(
            *statuses.lock().unwrap(),
            [
                UpdateStatus::Checking {
                    current_version: "1.2.3".to_owned(),
                },
                UpdateStatus::Available {
                    current_version: "1.2.3".to_owned(),
                    available_version: "2.0.0".to_owned(),
                },
            ]
        );
    }

    #[tokio::test]
    async fn backend_errors_are_sanitized_for_the_renderer() {
        let backend = Arc::new(StaticBackend::new(Err(UpdateBackendError::new(
            "network_error",
        ))));
        let (runtime, _) = runtime(backend, true);

        assert_eq!(
            runtime.check_for_updates().await.unwrap(),
            UpdateStatus::Error {
                current_version: "1.2.3".to_owned(),
                message: "Unable to check for updates.".to_owned(),
            }
        );
    }

    #[tokio::test]
    async fn unpackaged_checks_remain_unavailable() {
        let backend = Arc::new(StaticBackend::new(Ok(None)));
        let (runtime, _) = runtime(backend.clone(), false);

        assert_eq!(
            runtime.check_automatically().await.unwrap(),
            UpdateStatus::Idle {
                current_version: "1.2.3".to_owned(),
            }
        );
        assert_eq!(
            runtime.check_for_updates().await.unwrap(),
            UpdateStatus::Error {
                current_version: "1.2.3".to_owned(),
                message: "Update checks are available in packaged builds.".to_owned(),
            }
        );
        assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn concurrent_checks_share_one_backend_operation() {
        let backend = Arc::new(BlockingBackend {
            calls: AtomicUsize::new(0),
            started: Notify::new(),
            released: Notify::new(),
        });
        let (runtime, _) = runtime(backend.clone(), true);
        let first_runtime = runtime.clone();
        let first = tokio::spawn(async move { first_runtime.check_for_updates().await });

        backend.started.notified().await;
        let second_runtime = runtime.clone();
        let second = tokio::spawn(async move { second_runtime.check_for_updates().await });
        tokio::task::yield_now().await;
        assert_eq!(backend.calls.load(Ordering::SeqCst), 1);

        backend.released.notify_waiters();
        let first_status = first.await.unwrap().unwrap();
        let second_status = second.await.unwrap().unwrap();

        assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
        assert_eq!(first_status, second_status);
    }

    #[test]
    fn status_serialization_matches_the_shared_typescript_contract() {
        assert_eq!(
            serde_json::to_value(UpdateStatus::Available {
                current_version: "1.2.3".to_owned(),
                available_version: "2.0.0".to_owned(),
            })
            .unwrap(),
            serde_json::json!({
                "phase": "available",
                "currentVersion": "1.2.3",
                "availableVersion": "2.0.0"
            })
        );
        assert_eq!(
            serde_json::to_value(UpdateStatus::NotAvailable {
                current_version: "1.2.3".to_owned(),
            })
            .unwrap(),
            serde_json::json!({
                "phase": "not-available",
                "currentVersion": "1.2.3"
            })
        );
    }
}
