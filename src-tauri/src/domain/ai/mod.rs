// Provider milestones make the registry runtime-reachable incrementally.
// Keep the completed registry contract compiled and tested between milestones.
mod claude;
mod custom;
mod gemini;
mod grok;
mod ollama;
mod openai;
#[allow(dead_code)]
mod provider;
#[allow(dead_code)]
mod registry;
mod runtime;

pub(crate) use claude::ClaudeProvider;
pub(crate) use custom::CustomProvider;
pub(crate) use gemini::GeminiProvider;
pub(crate) use grok::GrokProvider;
pub(crate) use ollama::OllamaProvider;
pub(crate) use openai::OpenAiProvider;
pub(crate) use provider::{
    AiProvider, AiProviderConfiguration, AiProviderError, AiProviderErrorCode, AiRequest,
    AiResponse,
};
pub(crate) use registry::{AiProviderId, AiProviderRegistry, AiRegistryError};
pub(crate) use runtime::AiRuntime;
