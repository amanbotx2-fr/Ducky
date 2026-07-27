// Provider milestones make the registry runtime-reachable incrementally.
// Keep the completed registry contract compiled and tested between milestones.
#[allow(dead_code)]
mod registry;
mod runtime;

pub(crate) use registry::{AiProvider, AiProviderId, AiProviderRegistry, AiRegistryError};
pub(crate) use runtime::AiRuntime;
