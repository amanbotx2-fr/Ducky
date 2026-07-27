// Provider milestones make the registry runtime-reachable incrementally.
// Keep the completed registry contract compiled and tested between milestones.
mod openai;
#[allow(dead_code)]
mod provider;
#[allow(dead_code)]
mod registry;
mod runtime;

pub(crate) use openai::OpenAiProvider;
pub(crate) use provider::{
    AiProvider, AiProviderConfiguration, AiProviderError, AiProviderErrorCode, AiRequest,
    AiResponse,
};
pub(crate) use registry::{AiProviderId, AiProviderRegistry, AiRegistryError};
pub(crate) use runtime::AiRuntime;
