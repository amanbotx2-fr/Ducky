use std::sync::{Arc, Mutex, PoisonError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AiRuntimeError {
    Unavailable,
    ShuttingDown,
}

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
#[derive(Clone, Debug, Default)]
pub(crate) struct AiRuntime {
    state: Arc<Mutex<AiRuntimeState>>,
}

impl AiRuntime {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn ensure_running(&self) -> Result<(), AiRuntimeError> {
        if self.state.lock()?.shutting_down {
            Err(AiRuntimeError::ShuttingDown)
        } else {
            Ok(())
        }
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
    use super::*;

    #[test]
    fn runtime_initializes_available_and_shuts_down_cleanly() {
        let runtime = AiRuntime::new();

        assert_eq!(runtime.ensure_running(), Ok(()));
        assert_eq!(runtime.is_shutting_down(), Ok(false));
        assert_eq!(runtime.shutdown(), Ok(()));
        assert_eq!(runtime.is_shutting_down(), Ok(true));
        assert_eq!(runtime.ensure_running(), Err(AiRuntimeError::ShuttingDown));
    }

    #[test]
    fn cloned_handles_share_one_runtime_lifecycle() {
        let runtime = AiRuntime::new();
        let clone = runtime.clone();

        clone.shutdown().unwrap();

        assert_eq!(runtime.ensure_running(), Err(AiRuntimeError::ShuttingDown));
    }
}
