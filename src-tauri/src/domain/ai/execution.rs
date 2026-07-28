use std::{
    collections::{HashMap, VecDeque},
    future::Future,
    sync::{Arc, Mutex, PoisonError},
    time::{Duration, Instant},
};

use tokio::sync::watch;

const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum AiRendererRole {
    Companion,
    Preferences,
}

impl AiRendererRole {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Companion => "companion",
            Self::Preferences => "preferences",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum AiOperation {
    Chat,
    ConnectionTest,
    ModelDiscovery,
}

impl AiOperation {
    const fn maximum_requests(self) -> usize {
        match self {
            Self::Chat => 30,
            Self::ConnectionTest | Self::ModelDiscovery => 12,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Chat => "chat",
            Self::ConnectionTest => "connection_test",
            Self::ModelDiscovery => "model_discovery",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AiCancellationReason {
    ApplicationQuit,
    ProviderChanged,
    RendererReloaded,
    WindowClosed,
}

impl AiCancellationReason {
    const fn as_str(self) -> &'static str {
        match self {
            Self::ApplicationQuit => "application_quit",
            Self::ProviderChanged => "provider_changed",
            Self::RendererReloaded => "renderer_reloaded",
            Self::WindowClosed => "window_closed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AiRequestPolicyError {
    Cancelled,
    InProgress,
    RateLimited,
    Unavailable,
}

impl AiRequestPolicyError {
    pub(crate) const fn message(self) -> &'static str {
        match self {
            Self::Cancelled => "The AI request was cancelled.",
            Self::InProgress => "Request already in progress.",
            Self::RateLimited => "Too many AI requests. Try again shortly.",
            Self::Unavailable => "Chat is unavailable right now.",
        }
    }
}

impl std::fmt::Display for AiRequestPolicyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for AiRequestPolicyError {}

#[derive(Debug)]
struct ActiveOperation {
    id: u64,
    operation: AiOperation,
    cancellation: watch::Sender<bool>,
}

#[derive(Debug, Default)]
struct AiRequestState {
    active_by_role: HashMap<AiRendererRole, ActiveOperation>,
    request_timestamps: HashMap<(AiRendererRole, AiOperation), VecDeque<Instant>>,
    next_operation_id: u64,
}

/// Owns the application per-renderer execution policy:
/// `AIRequestManager`.
///
/// Cancellation stays entirely native. Dropping the selected provider future
/// cancels the in-flight reqwest operation without exposing a renderer command.
#[derive(Clone, Debug, Default)]
pub(crate) struct AiRequestManager {
    state: Arc<Mutex<AiRequestState>>,
}

impl AiRequestManager {
    pub(crate) async fn run<Output, OperationFuture>(
        &self,
        renderer_role: AiRendererRole,
        operation: AiOperation,
        execute: OperationFuture,
    ) -> Result<Output, AiRequestPolicyError>
    where
        OperationFuture: Future<Output = Output>,
    {
        let (operation_id, mut cancellation) = self.begin(renderer_role, operation)?;

        let result = tokio::select! {
            biased;
            cancellation_result = cancellation.changed() => {
                let _ = cancellation_result;
                Err(AiRequestPolicyError::Cancelled)
            }
            output = execute => Ok(output),
        };

        self.finish(renderer_role, operation_id);
        result
    }

    pub(crate) fn cancel_role(&self, renderer_role: AiRendererRole, reason: AiCancellationReason) {
        let state = match self.state.lock() {
            Ok(state) => state,
            Err(_) => {
                eprintln!("[security] ai_operation_cancel_failed: request manager unavailable");
                return;
            }
        };
        let Some(active) = state.active_by_role.get(&renderer_role) else {
            return;
        };

        eprintln!(
            "[security] ai_operation_cancelled: operation={} renderer_role={} reason={}",
            active.operation.as_str(),
            renderer_role.as_str(),
            reason.as_str()
        );
        let _ = active.cancellation.send(true);
    }

    pub(crate) fn cancel_all(&self, reason: AiCancellationReason) {
        self.cancel_role(AiRendererRole::Companion, reason);
        self.cancel_role(AiRendererRole::Preferences, reason);
    }

    fn begin(
        &self,
        renderer_role: AiRendererRole,
        operation: AiOperation,
    ) -> Result<(u64, watch::Receiver<bool>), AiRequestPolicyError> {
        let mut state = self.state.lock()?;
        if state.active_by_role.contains_key(&renderer_role) {
            log_rejection(renderer_role, operation, "operation_in_progress");
            return Err(AiRequestPolicyError::InProgress);
        }

        let now = Instant::now();
        let timestamps = state
            .request_timestamps
            .entry((renderer_role, operation))
            .or_default();
        while timestamps
            .front()
            .is_some_and(|timestamp| now.duration_since(*timestamp) >= RATE_LIMIT_WINDOW)
        {
            timestamps.pop_front();
        }
        if timestamps.len() >= operation.maximum_requests() {
            log_rejection(renderer_role, operation, "rate_limit_exceeded");
            return Err(AiRequestPolicyError::RateLimited);
        }
        timestamps.push_back(now);

        state.next_operation_id = state.next_operation_id.wrapping_add(1);
        let operation_id = state.next_operation_id;
        let (cancellation, receiver) = watch::channel(false);
        state.active_by_role.insert(
            renderer_role,
            ActiveOperation {
                id: operation_id,
                operation,
                cancellation,
            },
        );
        Ok((operation_id, receiver))
    }

    fn finish(&self, renderer_role: AiRendererRole, operation_id: u64) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if state
            .active_by_role
            .get(&renderer_role)
            .is_some_and(|active| active.id == operation_id)
        {
            state.active_by_role.remove(&renderer_role);
        }
    }
}

impl<T> From<PoisonError<T>> for AiRequestPolicyError {
    fn from(_error: PoisonError<T>) -> Self {
        Self::Unavailable
    }
}

fn log_rejection(renderer_role: AiRendererRole, operation: AiOperation, reason: &'static str) {
    eprintln!(
        "[security] ai_operation_rejected: operation={} renderer_role={} reason={reason}",
        operation.as_str(),
        renderer_role.as_str()
    );
}

#[cfg(test)]
mod tests {
    use std::{
        future::pending,
        sync::atomic::{AtomicUsize, Ordering},
    };

    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn one_active_request_is_allowed_per_renderer_role() {
        let manager = AiRequestManager::default();
        let first_manager = manager.clone();
        let first = tokio::spawn(async move {
            first_manager
                .run(
                    AiRendererRole::Companion,
                    AiOperation::Chat,
                    pending::<()>(),
                )
                .await
        });
        tokio::task::yield_now().await;

        assert_eq!(
            manager
                .run(AiRendererRole::Companion, AiOperation::Chat, async {})
                .await,
            Err(AiRequestPolicyError::InProgress)
        );
        assert_eq!(
            manager
                .run(
                    AiRendererRole::Preferences,
                    AiOperation::ConnectionTest,
                    async { "preferences" },
                )
                .await,
            Ok("preferences")
        );

        manager.cancel_role(
            AiRendererRole::Companion,
            AiCancellationReason::RendererReloaded,
        );
        assert_eq!(first.await.unwrap(), Err(AiRequestPolicyError::Cancelled));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn rate_limits_apply_per_role_and_operation() {
        let manager = AiRequestManager::default();

        for request in 0..30 {
            assert_eq!(
                manager
                    .run(AiRendererRole::Companion, AiOperation::Chat, async move {
                        request
                    },)
                    .await,
                Ok(request)
            );
        }
        assert_eq!(
            manager
                .run(AiRendererRole::Companion, AiOperation::Chat, async {})
                .await,
            Err(AiRequestPolicyError::RateLimited)
        );

        for _ in 0..12 {
            assert_eq!(
                manager
                    .run(
                        AiRendererRole::Preferences,
                        AiOperation::ConnectionTest,
                        async {},
                    )
                    .await,
                Ok(())
            );
        }
        assert_eq!(
            manager
                .run(
                    AiRendererRole::Preferences,
                    AiOperation::ConnectionTest,
                    async {},
                )
                .await,
            Err(AiRequestPolicyError::RateLimited)
        );
        assert_eq!(
            manager
                .run(
                    AiRendererRole::Preferences,
                    AiOperation::ModelDiscovery,
                    async { "separate operation" },
                )
                .await,
            Ok("separate operation")
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn lifecycle_cancellation_drops_provider_work_and_cleans_up() {
        struct DropGuard(Arc<AtomicUsize>);

        impl Drop for DropGuard {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }

        let manager = AiRequestManager::default();
        let dropped = Arc::new(AtomicUsize::new(0));
        let first_manager = manager.clone();
        let first_dropped = dropped.clone();
        let first = tokio::spawn(async move {
            first_manager
                .run(AiRendererRole::Companion, AiOperation::Chat, async move {
                    let _guard = DropGuard(first_dropped);
                    pending::<()>().await;
                })
                .await
        });
        tokio::task::yield_now().await;

        manager.cancel_all(AiCancellationReason::ProviderChanged);
        assert_eq!(first.await.unwrap(), Err(AiRequestPolicyError::Cancelled));
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
        assert_eq!(
            manager
                .run(AiRendererRole::Companion, AiOperation::Chat, async {
                    "next request"
                },)
                .await,
            Ok("next request")
        );
    }
}
