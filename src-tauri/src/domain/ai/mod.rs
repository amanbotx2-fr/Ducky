// Provider milestones make the registry runtime-reachable incrementally.
// Keep the completed registry contract compiled and tested between milestones.
mod actions;
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

pub(crate) use actions::{AiConversationRequest, AssistantActionError, AssistantActionProcessor};
pub(crate) use claude::ClaudeProvider;
pub(crate) use custom::CustomProvider;
pub(crate) use gemini::GeminiProvider;
pub(crate) use grok::GrokProvider;
pub(crate) use ollama::is_valid_endpoint as is_valid_ollama_endpoint;
pub(crate) use ollama::OllamaProvider;
pub(crate) use openai::OpenAiProvider;
pub(crate) use provider::{
    AiFinishReason, AiModel, AiProvider, AiProviderConfiguration, AiProviderError,
    AiProviderErrorCode, AiProviderHttpDiagnostics, AiRequest, AiResponse,
};
pub(crate) use registry::{AiProviderId, AiProviderRegistry, AiRegistryError};
pub(crate) use runtime::{AiExecutionError, AiRuntime};
