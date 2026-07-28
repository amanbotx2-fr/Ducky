use std::time::Duration;

use async_trait::async_trait;
use tauri::{AppHandle, Runtime};
use tauri_plugin_updater::{Error as TauriUpdaterError, UpdaterExt};

use crate::domain::updater::{UpdateBackend, UpdateBackendError, UpdateRelease};

const UPDATE_CHECK_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) struct TauriUpdateBackend<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> TauriUpdateBackend<R> {
    pub(crate) fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl<R: Runtime> UpdateBackend for TauriUpdateBackend<R> {
    async fn check(&self) -> Result<Option<UpdateRelease>, UpdateBackendError> {
        let updater = self
            .app
            .updater_builder()
            .timeout(UPDATE_CHECK_TIMEOUT)
            .version_comparator(|current, release| {
                release.version > current && release.version.pre.is_empty()
            })
            .build()
            .map_err(map_updater_error)?;
        updater
            .check()
            .await
            .map(|release| {
                release.map(|update| UpdateRelease {
                    version: update.version,
                })
            })
            .map_err(map_updater_error)
    }
}

fn map_updater_error(error: TauriUpdaterError) -> UpdateBackendError {
    let code = match error {
        TauriUpdaterError::EmptyEndpoints => "empty_endpoints",
        TauriUpdaterError::ReleaseNotFound => "release_not_found",
        TauriUpdaterError::UnsupportedArch => "unsupported_arch",
        TauriUpdaterError::UnsupportedOs => "unsupported_os",
        TauriUpdaterError::InsecureTransportProtocol => "insecure_transport",
        TauriUpdaterError::Http(_)
        | TauriUpdaterError::Network(_)
        | TauriUpdaterError::Reqwest(_) => "network_error",
        _ => "updater_error",
    };

    UpdateBackendError::new(code)
}
