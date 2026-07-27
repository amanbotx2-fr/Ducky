use std::{
    collections::BTreeMap,
    sync::{Arc, PoisonError, RwLock},
};

use serde::{Deserialize, Serialize};

use super::provider::AiProvider;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AiProviderId {
    Openai,
    Gemini,
    Claude,
    Grok,
    Ollama,
    Custom,
}

impl AiProviderId {
    pub(crate) const ALL: [Self; 6] = [
        Self::Openai,
        Self::Gemini,
        Self::Claude,
        Self::Grok,
        Self::Ollama,
        Self::Custom,
    ];

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::Gemini => "gemini",
            Self::Claude => "claude",
            Self::Grok => "grok",
            Self::Ollama => "ollama",
            Self::Custom => "custom",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "openai" => Some(Self::Openai),
            "gemini" => Some(Self::Gemini),
            "claude" => Some(Self::Claude),
            "grok" => Some(Self::Grok),
            "ollama" => Some(Self::Ollama),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

impl std::fmt::Display for AiProviderId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AiRegistryError {
    DuplicateProvider,
    UnknownProvider,
    Unavailable,
}

impl std::fmt::Display for AiRegistryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateProvider => formatter.write_str("AI provider is already registered"),
            Self::UnknownProvider => formatter.write_str("AI provider is not registered"),
            Self::Unavailable => formatter.write_str("AI provider registry is unavailable"),
        }
    }
}

impl std::error::Error for AiRegistryError {}

#[derive(Default)]
struct ProviderRegistryState {
    providers: BTreeMap<AiProviderId, Arc<dyn AiProvider>>,
    active_provider: Option<AiProviderId>,
}

impl std::fmt::Debug for ProviderRegistryState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProviderRegistryState")
            .field("provider_ids", &self.providers.keys())
            .field("active_provider", &self.active_provider)
            .finish()
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct AiProviderRegistry {
    state: Arc<RwLock<ProviderRegistryState>>,
}

impl AiProviderRegistry {
    pub(crate) fn register(&self, provider: Arc<dyn AiProvider>) -> Result<(), AiRegistryError> {
        let id = provider.id();
        let mut state = self.state.write()?;
        if state.providers.contains_key(&id) {
            return Err(AiRegistryError::DuplicateProvider);
        }

        state.providers.insert(id, provider);
        Ok(())
    }

    pub(crate) fn select(&self, id: AiProviderId) -> Result<(), AiRegistryError> {
        let mut state = self.state.write()?;
        if !state.providers.contains_key(&id) {
            return Err(AiRegistryError::UnknownProvider);
        }

        state.active_provider = Some(id);
        Ok(())
    }

    pub(crate) fn clear_selection(&self) -> Result<(), AiRegistryError> {
        self.state.write()?.active_provider = None;
        Ok(())
    }

    pub(crate) fn provider(
        &self,
        id: AiProviderId,
    ) -> Result<Arc<dyn AiProvider>, AiRegistryError> {
        self.state
            .read()?
            .providers
            .get(&id)
            .cloned()
            .ok_or(AiRegistryError::UnknownProvider)
    }

    pub(crate) fn active_provider(&self) -> Result<Option<Arc<dyn AiProvider>>, AiRegistryError> {
        let state = self.state.read()?;
        state
            .active_provider
            .map(|id| {
                state
                    .providers
                    .get(&id)
                    .cloned()
                    .ok_or(AiRegistryError::UnknownProvider)
            })
            .transpose()
    }

    pub(crate) fn registered_ids(&self) -> Result<Vec<AiProviderId>, AiRegistryError> {
        Ok(self.state.read()?.providers.keys().copied().collect())
    }
}

impl<T> From<PoisonError<T>> for AiRegistryError {
    fn from(_error: PoisonError<T>) -> Self {
        Self::Unavailable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestProvider {
        id: AiProviderId,
        name: &'static str,
    }

    impl AiProvider for TestProvider {
        fn id(&self) -> AiProviderId {
            self.id
        }

        fn display_name(&self) -> &'static str {
            self.name
        }
    }

    fn provider(id: AiProviderId, name: &'static str) -> Arc<dyn AiProvider> {
        Arc::new(TestProvider { id, name })
    }

    #[test]
    fn registry_registers_selects_and_resolves_providers() {
        let registry = AiProviderRegistry::default();
        registry
            .register(provider(AiProviderId::Openai, "OpenAI"))
            .unwrap();
        registry
            .register(provider(AiProviderId::Gemini, "Gemini"))
            .unwrap();

        assert_eq!(
            registry.registered_ids().unwrap(),
            [AiProviderId::Openai, AiProviderId::Gemini]
        );
        assert!(registry.active_provider().unwrap().is_none());

        registry.select(AiProviderId::Gemini).unwrap();
        let active = registry.active_provider().unwrap().unwrap();
        assert_eq!(active.id(), AiProviderId::Gemini);
        assert_eq!(active.display_name(), "Gemini");
    }

    #[test]
    fn registry_rejects_duplicates_and_unknown_selection() {
        let registry = AiProviderRegistry::default();
        registry
            .register(provider(AiProviderId::Openai, "OpenAI"))
            .unwrap();

        assert_eq!(
            registry.register(provider(AiProviderId::Openai, "Replacement")),
            Err(AiRegistryError::DuplicateProvider)
        );
        assert_eq!(
            registry.select(AiProviderId::Claude),
            Err(AiRegistryError::UnknownProvider)
        );
        assert!(matches!(
            registry.provider(AiProviderId::Claude),
            Err(AiRegistryError::UnknownProvider)
        ));
    }

    #[test]
    fn provider_identifiers_are_stable_and_complete() {
        assert_eq!(
            AiProviderId::ALL.map(AiProviderId::as_str),
            ["openai", "gemini", "claude", "grok", "ollama", "custom"]
        );
        for provider in AiProviderId::ALL {
            assert_eq!(AiProviderId::parse(provider.as_str()), Some(provider));
        }
        assert_eq!(AiProviderId::parse(" GEMINI "), Some(AiProviderId::Gemini));
        assert_eq!(AiProviderId::parse(""), None);
        assert_eq!(AiProviderId::parse("unsupported"), None);
    }
}
