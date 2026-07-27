use std::sync::{Arc, Mutex, PoisonError};

use super::{
    AiProvider, AiProviderConfiguration, AiProviderError, AiProviderId, AiProviderRegistry,
    AiRegistryError, AiRequest, AiResponse,
};
use crate::infrastructure::credentials::{CredentialId, CredentialStore, CredentialStoreError};
use zeroize::Zeroizing;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AiRuntimeError {
    Unavailable,
    ShuttingDown,
}

#[derive(Debug)]
pub(crate) enum AiExecutionError {
    Runtime(AiRuntimeError),
    Registry(AiRegistryError),
    Provider(AiProviderError),
    ProviderNotSelected,
}

impl std::fmt::Display for AiExecutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => error.fmt(formatter),
            Self::Registry(error) => error.fmt(formatter),
            Self::Provider(error) => error.fmt(formatter),
            Self::ProviderNotSelected => formatter.write_str("No AI provider is selected"),
        }
    }
}

impl std::error::Error for AiExecutionError {}

impl std::fmt::Display for AiRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => formatter.write_str("AI runtime is unavailable"),
            Self::ShuttingDown => formatter.write_str("AI runtime is shutting down"),
        }
    }
}

impl std::error::Error for AiRuntimeError {}

#[derive(Debug, Default)]
struct AiRuntimeState {
    shutting_down: bool,
}

/// Process-wide owner for the native AI domain.
///
/// Provider registration and execution are added by their owning Phase 9
/// milestones. Keeping lifecycle ownership independent from any provider
/// prevents renderer or transport concerns from becoming runtime state.
#[derive(Clone, Debug)]
pub(crate) struct AiRuntime {
    state: Arc<Mutex<AiRuntimeState>>,
    #[allow(dead_code)]
    providers: AiProviderRegistry,
    credentials: CredentialStore,
}

impl AiRuntime {
    pub(crate) fn new(credentials: CredentialStore) -> Self {
        Self {
            state: Arc::default(),
            providers: AiProviderRegistry::default(),
            credentials,
        }
    }

    pub(crate) fn ensure_running(&self) -> Result<(), AiRuntimeError> {
        if self.state.lock()?.shutting_down {
            Err(AiRuntimeError::ShuttingDown)
        } else {
            Ok(())
        }
    }

    #[allow(dead_code)]
    pub(crate) fn register_provider(
        &self,
        provider: Arc<dyn AiProvider>,
    ) -> Result<(), AiRegistryError> {
        self.providers.register(provider)
    }

    #[allow(dead_code)]
    pub(crate) fn select_provider(&self, id: AiProviderId) -> Result<(), AiRegistryError> {
        self.providers.select(id)
    }

    #[allow(dead_code)]
    pub(crate) fn active_provider(&self) -> Result<Option<Arc<dyn AiProvider>>, AiRegistryError> {
        self.providers.active_provider()
    }

    #[allow(dead_code)]
    pub(crate) fn registered_provider_ids(&self) -> Result<Vec<AiProviderId>, AiRegistryError> {
        self.providers.registered_ids()
    }

    #[allow(dead_code)]
    pub(crate) fn load_provider_credential(
        &self,
        provider: AiProviderId,
    ) -> Result<Option<Zeroizing<String>>, CredentialStoreError> {
        if provider == AiProviderId::Ollama {
            Ok(None)
        } else {
            self.credentials.load(CredentialId::AiApiKey)
        }
    }

    pub(crate) async fn send_message(
        &self,
        configuration: AiProviderConfiguration<'_>,
        request: AiRequest,
    ) -> Result<AiResponse, AiExecutionError> {
        self.ensure_running().map_err(AiExecutionError::Runtime)?;
        let provider = self
            .active_provider()
            .map_err(AiExecutionError::Registry)?
            .ok_or(AiExecutionError::ProviderNotSelected)?;
        provider
            .send_message(configuration, request)
            .await
            .map_err(AiExecutionError::Provider)
    }

    pub(crate) fn shutdown(&self) -> Result<(), AiRuntimeError> {
        self.state.lock()?.shutting_down = true;
        Ok(())
    }

    #[cfg(test)]
    fn is_shutting_down(&self) -> Result<bool, AiRuntimeError> {
        Ok(self.state.lock()?.shutting_down)
    }
}

impl<T> From<PoisonError<T>> for AiRuntimeError {
    fn from(_error: PoisonError<T>) -> Self {
        Self::Unavailable
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use super::*;
    use crate::domain::ai::{AiFinishReason, AiProviderErrorCode};

    #[derive(Debug)]
    struct EchoProvider;

    #[async_trait]
    impl AiProvider for EchoProvider {
        fn id(&self) -> AiProviderId {
            AiProviderId::Openai
        }

        fn display_name(&self) -> &'static str {
            "Echo"
        }

        async fn send_message(
            &self,
            _configuration: AiProviderConfiguration<'_>,
            request: AiRequest,
        ) -> Result<AiResponse, AiProviderError> {
            if request.prompt == "fail" {
                return Err(AiProviderError::new(
                    self.id(),
                    AiProviderErrorCode::Connection,
                    "failed",
                ));
            }
            Ok(AiResponse {
                provider_id: self.id(),
                content: request.prompt,
                finish_reason: AiFinishReason::Stop,
                usage: None,
            })
        }
    }

    #[test]
    fn runtime_initializes_available_and_shuts_down_cleanly() {
        let runtime = AiRuntime::new(CredentialStore::native());

        assert_eq!(runtime.ensure_running(), Ok(()));
        assert_eq!(runtime.is_shutting_down(), Ok(false));
        assert_eq!(runtime.shutdown(), Ok(()));
        assert_eq!(runtime.is_shutting_down(), Ok(true));
        assert_eq!(runtime.ensure_running(), Err(AiRuntimeError::ShuttingDown));
    }

    #[test]
    fn cloned_handles_share_one_runtime_lifecycle() {
        let runtime = AiRuntime::new(CredentialStore::native());
        let clone = runtime.clone();

        clone.shutdown().unwrap();

        assert_eq!(runtime.ensure_running(), Err(AiRuntimeError::ShuttingDown));
    }

    #[test]
    fn ollama_never_requests_a_cloud_provider_credential() {
        let runtime = AiRuntime::new(CredentialStore::native());

        assert_eq!(
            runtime
                .load_provider_credential(AiProviderId::Ollama)
                .unwrap(),
            None
        );
    }

    #[tokio::test]
    async fn execution_returns_one_complete_provider_response() {
        let runtime = AiRuntime::new(CredentialStore::native());
        runtime.register_provider(Arc::new(EchoProvider)).unwrap();
        runtime.select_provider(AiProviderId::Openai).unwrap();

        let response = runtime
            .send_message(
                AiProviderConfiguration {
                    api_key: None,
                    base_url: "",
                    endpoint: "",
                    model: "",
                },
                AiRequest {
                    prompt: "complete response".to_owned(),
                },
            )
            .await
            .unwrap();

        assert_eq!(response.content, "complete response");
        assert_eq!(response.finish_reason, AiFinishReason::Stop);
    }

    #[tokio::test]
    async fn execution_preserves_provider_failures_without_partial_output() {
        let runtime = AiRuntime::new(CredentialStore::native());
        runtime.register_provider(Arc::new(EchoProvider)).unwrap();
        runtime.select_provider(AiProviderId::Openai).unwrap();

        let error = runtime
            .send_message(
                AiProviderConfiguration {
                    api_key: None,
                    base_url: "",
                    endpoint: "",
                    model: "",
                },
                AiRequest {
                    prompt: "fail".to_owned(),
                },
            )
            .await
            .unwrap_err();

        assert!(matches!(error, AiExecutionError::Provider(_)));
    }
}
