use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex, PoisonError},
    thread::{self, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

pub(crate) const DOCUMENT_VERSION: u8 = 1;
pub(crate) const DEFAULT_DURATION_MINUTES: u32 = 25;
pub(crate) const MINIMUM_DURATION_MINUTES: u32 = 1;
pub(crate) const MAXIMUM_DURATION_MINUTES: u32 = 720;

const SECOND_MILLIS: i64 = 1_000;
const MAXIMUM_SAFE_INTEGER: i64 = 9_007_199_254_740_991;
const INVALID_DURATION_MESSAGE: &str = "Pomodoro duration must be between 1 and 720 minutes.";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PomodoroState {
    pub(crate) running: bool,
    pub(crate) paused: bool,
    pub(crate) selected_duration_minutes: u32,
    pub(crate) duration_minutes: u32,
    pub(crate) remaining_seconds: u32,
    pub(crate) started_at: Option<i64>,
}

impl PomodoroState {
    pub(crate) fn idle(selected_duration_minutes: u32) -> Result<Self, PomodoroError> {
        validate_duration(selected_duration_minutes)?;

        Ok(Self {
            running: false,
            paused: false,
            selected_duration_minutes,
            duration_minutes: selected_duration_minutes,
            remaining_seconds: 0,
            started_at: None,
        })
    }

    pub(crate) fn validate(&self) -> Result<(), PomodoroError> {
        validate_duration(self.selected_duration_minutes)?;
        validate_duration(self.duration_minutes)?;

        if self.remaining_seconds > self.duration_minutes.saturating_mul(60) {
            return Err(PomodoroError::InvalidState);
        }

        if let Some(started_at) = self.started_at {
            if !(0..=MAXIMUM_SAFE_INTEGER).contains(&started_at) {
                return Err(PomodoroError::InvalidState);
            }
        }

        if (!self.running
            && (self.paused || self.remaining_seconds != 0 || self.started_at.is_some()))
            || (self.running && self.remaining_seconds == 0)
            || (self.paused && !self.running)
            || (self.running && !self.paused && self.started_at.is_none())
        {
            return Err(PomodoroError::InvalidState);
        }

        Ok(())
    }
}

impl Default for PomodoroState {
    fn default() -> Self {
        Self::idle(DEFAULT_DURATION_MINUTES).expect("default Pomodoro duration is valid")
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PersistedPomodoroDocument {
    pub(crate) version: u8,
    pub(crate) state: PomodoroState,
}

impl PersistedPomodoroDocument {
    pub(crate) fn new(state: PomodoroState) -> Self {
        Self {
            version: DOCUMENT_VERSION,
            state,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), PomodoroError> {
        if self.version != DOCUMENT_VERSION {
            return Err(PomodoroError::InvalidDocument);
        }

        self.state.validate()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PomodoroLoad {
    Missing,
    Invalid,
    Document(PersistedPomodoroDocument),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PomodoroRepositoryError {
    message: String,
}

impl PomodoroRepositoryError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for PomodoroRepositoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for PomodoroRepositoryError {}

pub(crate) trait PomodoroRepository: Send + Sync {
    fn load(&self) -> Result<PomodoroLoad, PomodoroRepositoryError>;
    fn save(&self, document: &PersistedPomodoroDocument) -> Result<(), PomodoroRepositoryError>;
}

pub(crate) trait PomodoroEvents: Send + Sync {
    fn state_changed(&self, state: PomodoroState);
    fn completed(&self);
}

#[derive(Debug, Default)]
struct PendingEvents {
    latest_state: Option<PomodoroState>,
    pending_completion: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PomodoroEventQueue {
    pending: Arc<Mutex<PendingEvents>>,
}

impl PomodoroEventQueue {
    pub(crate) fn latest_state(&self) -> Result<Option<PomodoroState>, PomodoroError> {
        Ok(self.pending.lock()?.latest_state.clone())
    }

    pub(crate) fn has_pending_completion(&self) -> Result<bool, PomodoroError> {
        Ok(self.pending.lock()?.pending_completion)
    }
}

impl PomodoroEvents for PomodoroEventQueue {
    fn state_changed(&self, state: PomodoroState) {
        match self.pending.lock() {
            Ok(mut pending) => {
                pending.latest_state = Some(state);
            }
            Err(_) => {
                eprintln!("[pomodoro] state_event_queue_unavailable");
            }
        }
    }

    fn completed(&self) {
        match self.pending.lock() {
            Ok(mut pending) => {
                pending.pending_completion = true;
            }
            Err(_) => {
                eprintln!("[pomodoro] completion_event_queue_unavailable");
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PomodoroError {
    InvalidDuration,
    InvalidState,
    InvalidDocument,
    StateUnavailable,
    RuntimeUnavailable,
}

impl std::fmt::Display for PomodoroError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDuration => formatter.write_str(INVALID_DURATION_MESSAGE),
            Self::InvalidState => formatter.write_str("Pomodoro state is invalid."),
            Self::InvalidDocument => formatter.write_str("Pomodoro document is invalid."),
            Self::StateUnavailable => formatter.write_str("Pomodoro state is unavailable."),
            Self::RuntimeUnavailable => formatter.write_str("Pomodoro runtime is unavailable."),
        }
    }
}

impl std::error::Error for PomodoroError {}

impl<T> From<PoisonError<T>> for PomodoroError {
    fn from(_error: PoisonError<T>) -> Self {
        Self::StateUnavailable
    }
}

fn validate_duration(duration_minutes: u32) -> Result<(), PomodoroError> {
    if (MINIMUM_DURATION_MINUTES..=MAXIMUM_DURATION_MINUTES).contains(&duration_minutes) {
        Ok(())
    } else {
        Err(PomodoroError::InvalidDuration)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EngineResult {
    state: PomodoroState,
    state_changed: bool,
    completed: bool,
}

#[derive(Debug)]
struct PomodoroEngine {
    state: PomodoroState,
}

impl PomodoroEngine {
    fn new(state: PomodoroState) -> Result<Self, PomodoroError> {
        state.validate()?;
        Ok(Self { state })
    }

    fn snapshot(&self, now_millis: i64) -> PomodoroState {
        materialize_state(&self.state, now_millis)
    }

    fn reconcile(&mut self, now_millis: i64) -> EngineResult {
        let snapshot = self.snapshot(now_millis);

        if snapshot.running && snapshot.remaining_seconds == 0 {
            self.complete()
        } else {
            EngineResult {
                state: snapshot,
                state_changed: false,
                completed: false,
            }
        }
    }

    fn start(
        &mut self,
        duration_minutes: u32,
        now_millis: i64,
    ) -> Result<EngineResult, PomodoroError> {
        validate_duration(duration_minutes)?;
        self.state = PomodoroState {
            running: true,
            paused: false,
            selected_duration_minutes: duration_minutes,
            duration_minutes,
            remaining_seconds: duration_minutes.saturating_mul(60),
            started_at: Some(now_millis),
        };

        Ok(self.changed())
    }

    fn pause(&mut self, now_millis: i64) -> EngineResult {
        if !self.state.running || self.state.paused {
            return self.unchanged(now_millis);
        }

        let snapshot = self.snapshot(now_millis);
        if snapshot.remaining_seconds == 0 {
            return self.complete();
        }

        self.state = PomodoroState {
            paused: true,
            ..snapshot
        };
        self.changed()
    }

    fn resume(&mut self, now_millis: i64) -> EngineResult {
        if !self.state.running || !self.state.paused {
            return self.unchanged(now_millis);
        }

        self.state.paused = false;
        self.state.started_at = Some(now_millis);
        self.changed()
    }

    fn stop(&mut self) -> Result<EngineResult, PomodoroError> {
        if !self.state.running {
            return Ok(EngineResult {
                state: self.state.clone(),
                state_changed: false,
                completed: false,
            });
        }

        self.state = PomodoroState::idle(self.state.selected_duration_minutes)?;
        Ok(self.changed())
    }

    #[cfg(test)]
    fn set_duration(
        &mut self,
        duration_minutes: u32,
        now_millis: i64,
    ) -> Result<EngineResult, PomodoroError> {
        validate_duration(duration_minutes)?;
        let current = self.snapshot(now_millis);

        if duration_minutes == current.selected_duration_minutes {
            return Ok(EngineResult {
                state: current,
                state_changed: false,
                completed: false,
            });
        }

        self.state = if current.running {
            PomodoroState {
                selected_duration_minutes: duration_minutes,
                ..self.state.clone()
            }
        } else {
            PomodoroState::idle(duration_minutes)?
        };

        Ok(self.changed())
    }

    fn complete(&mut self) -> EngineResult {
        let selected_duration_minutes = self.state.selected_duration_minutes;
        self.state = PomodoroState::idle(selected_duration_minutes)
            .expect("selected Pomodoro duration was previously validated");

        EngineResult {
            state: self.state.clone(),
            state_changed: true,
            completed: true,
        }
    }

    fn changed(&self) -> EngineResult {
        EngineResult {
            state: self.state.clone(),
            state_changed: true,
            completed: false,
        }
    }

    fn unchanged(&self, now_millis: i64) -> EngineResult {
        EngineResult {
            state: self.snapshot(now_millis),
            state_changed: false,
            completed: false,
        }
    }
}

fn materialize_state(state: &PomodoroState, now_millis: i64) -> PomodoroState {
    if !state.running || state.paused {
        return state.clone();
    }

    let Some(started_at) = state.started_at else {
        return state.clone();
    };
    let elapsed_millis = now_millis.saturating_sub(started_at).max(0);
    let elapsed_seconds = u32::try_from(elapsed_millis / SECOND_MILLIS).unwrap_or(u32::MAX);

    PomodoroState {
        remaining_seconds: state.remaining_seconds.saturating_sub(elapsed_seconds),
        ..state.clone()
    }
}

type Clock = Arc<dyn Fn() -> i64 + Send + Sync>;

#[derive(Debug, Default)]
struct SchedulerSignal {
    generation: u64,
    shutdown: bool,
}

#[derive(Debug, Default)]
struct SchedulerWake {
    signal: Mutex<SchedulerSignal>,
    changed: Condvar,
}

impl SchedulerWake {
    fn notify(&self) {
        if let Ok(mut signal) = self.signal.lock() {
            signal.generation = signal.generation.wrapping_add(1);
            self.changed.notify_all();
        }
    }

    fn shutdown(&self) {
        if let Ok(mut signal) = self.signal.lock() {
            signal.shutdown = true;
            signal.generation = signal.generation.wrapping_add(1);
            self.changed.notify_all();
        }
    }
}

struct RuntimeCore {
    repository: Arc<dyn PomodoroRepository>,
    events: Arc<dyn PomodoroEvents>,
    engine: Mutex<PomodoroEngine>,
    operations: Mutex<()>,
    pending_saves: Mutex<VecDeque<PersistedPomodoroDocument>>,
    wake: SchedulerWake,
    now: Clock,
}

impl std::fmt::Debug for RuntimeCore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeCore")
            .field("repository", &"<repository>")
            .field("events", &"<events>")
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub(crate) struct PomodoroRuntime {
    core: Arc<RuntimeCore>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl PomodoroRuntime {
    pub(crate) fn new(
        repository: Arc<dyn PomodoroRepository>,
        events: Arc<dyn PomodoroEvents>,
    ) -> Self {
        Self::with_clock(repository, events, Arc::new(current_timestamp_millis))
    }

    fn with_clock(
        repository: Arc<dyn PomodoroRepository>,
        events: Arc<dyn PomodoroEvents>,
        now: Clock,
    ) -> Self {
        Self {
            core: Arc::new(RuntimeCore {
                repository,
                events,
                engine: Mutex::new(
                    PomodoroEngine::new(PomodoroState::default())
                        .expect("default Pomodoro state is valid"),
                ),
                operations: Mutex::new(()),
                pending_saves: Mutex::new(VecDeque::new()),
                wake: SchedulerWake::default(),
                now,
            }),
            worker: Mutex::new(None),
        }
    }

    pub(crate) fn start(&self) -> Result<PomodoroState, PomodoroError> {
        let mut worker = self.worker.lock()?;
        if worker.is_some() {
            return self.state();
        }

        let initial_state = self.load()?;
        let core = Arc::clone(&self.core);
        *worker = Some(
            thread::Builder::new()
                .name("ducky-pomodoro".to_owned())
                .spawn(move || run_scheduler(core))
                .map_err(|_| PomodoroError::RuntimeUnavailable)?,
        );
        Ok(initial_state)
    }

    pub(crate) fn state(&self) -> Result<PomodoroState, PomodoroError> {
        let _operation = self.core.operations.lock()?;
        let result = self.core.engine.lock()?.reconcile((self.core.now)());
        self.publish_result(&result, result.state_changed);
        Ok(result.state)
    }

    pub(crate) fn start_session(
        &self,
        duration_minutes: u32,
    ) -> Result<PomodoroState, PomodoroError> {
        let _operation = self.core.operations.lock()?;
        let result = self
            .core
            .engine
            .lock()?
            .start(duration_minutes, (self.core.now)())?;
        self.publish_result(&result, true);
        self.core.wake.notify();
        Ok(result.state)
    }

    pub(crate) fn pause(&self) -> Result<PomodoroState, PomodoroError> {
        let _operation = self.core.operations.lock()?;
        let result = self.core.engine.lock()?.pause((self.core.now)());
        self.publish_result(&result, result.state_changed);
        self.core.wake.notify();
        Ok(result.state)
    }

    pub(crate) fn resume(&self) -> Result<PomodoroState, PomodoroError> {
        let _operation = self.core.operations.lock()?;
        let result = self.core.engine.lock()?.resume((self.core.now)());
        self.publish_result(&result, result.state_changed);
        self.core.wake.notify();
        Ok(result.state)
    }

    pub(crate) fn stop_session(&self) -> Result<PomodoroState, PomodoroError> {
        let _operation = self.core.operations.lock()?;
        let result = self.core.engine.lock()?.stop()?;
        self.publish_result(&result, result.state_changed);
        self.core.wake.notify();
        Ok(result.state)
    }

    pub(crate) fn stop(&self) -> Result<(), PomodoroError> {
        self.core.wake.shutdown();
        let worker = self.worker.lock()?.take();

        if let Some(worker) = worker {
            worker
                .join()
                .map_err(|_| PomodoroError::RuntimeUnavailable)?;
        }

        Ok(())
    }

    fn load(&self) -> Result<PomodoroState, PomodoroError> {
        let _operation = self.core.operations.lock()?;
        let state = match self.core.repository.load() {
            Ok(PomodoroLoad::Document(document)) => {
                if document.validate().is_ok() {
                    document.state
                } else {
                    eprintln!("[pomodoro] invalid_persisted_state");
                    PomodoroState::default()
                }
            }
            Ok(PomodoroLoad::Missing) => {
                let state = PomodoroState::default();
                self.save_immediately(&PersistedPomodoroDocument::new(state.clone()));
                state
            }
            Ok(PomodoroLoad::Invalid) => {
                eprintln!("[pomodoro] invalid_persisted_state");
                PomodoroState::default()
            }
            Err(error) => {
                eprintln!("[pomodoro] load_failed: {error}");
                PomodoroState::default()
            }
        };

        *self.core.engine.lock()? = PomodoroEngine::new(state)?;
        let result = self.core.engine.lock()?.reconcile((self.core.now)());

        if result.state_changed {
            self.save_immediately(&PersistedPomodoroDocument::new(result.state.clone()));
        }

        self.core.events.state_changed(result.state.clone());
        if result.completed {
            self.core.events.completed();
        }

        Ok(result.state)
    }

    fn publish_result(&self, result: &EngineResult, persist: bool) {
        if persist {
            match self.core.pending_saves.lock() {
                Ok(mut pending) => {
                    pending.push_back(PersistedPomodoroDocument::new(result.state.clone()));
                }
                Err(_) => {
                    eprintln!("[pomodoro] save_failed: persistence queue unavailable");
                }
            }
        }

        if result.state_changed {
            self.core.events.state_changed(result.state.clone());
        }
        if result.completed {
            self.core.events.completed();
        }
    }

    fn save_immediately(&self, document: &PersistedPomodoroDocument) {
        if let Err(error) = self.core.repository.save(document) {
            eprintln!("[pomodoro] save_failed: {error}");
        }
    }
}

impl Drop for PomodoroRuntime {
    fn drop(&mut self) {
        self.core.wake.shutdown();
    }
}

fn run_scheduler(core: Arc<RuntimeCore>) {
    loop {
        flush_pending_saves(&core);

        let shutdown = core
            .wake
            .signal
            .lock()
            .map(|signal| signal.shutdown)
            .unwrap_or(true);
        if shutdown {
            flush_pending_saves(&core);
            return;
        }

        let delay = reconcile_runtime(&core);
        let mut signal = match core.wake.signal.lock() {
            Ok(signal) => signal,
            Err(_) => return,
        };
        if signal.shutdown {
            continue;
        }
        let generation = signal.generation;

        match delay {
            Some(delay) => {
                let waited = core
                    .wake
                    .changed
                    .wait_timeout_while(signal, delay, |current| {
                        !current.shutdown && current.generation == generation
                    });
                if waited.is_err() {
                    return;
                }
            }
            None => {
                while !signal.shutdown && signal.generation == generation {
                    signal = match core.wake.changed.wait(signal) {
                        Ok(signal) => signal,
                        Err(_) => return,
                    };
                }
            }
        }
    }
}

fn reconcile_runtime(core: &RuntimeCore) -> Option<Duration> {
    let _operation = core.operations.lock().ok()?;
    let now_millis = (core.now)();
    let result = core.engine.lock().ok()?.reconcile(now_millis);

    if result.state_changed {
        if let Ok(mut pending) = core.pending_saves.lock() {
            pending.push_back(PersistedPomodoroDocument::new(result.state.clone()));
        } else {
            eprintln!("[pomodoro] save_failed: persistence queue unavailable");
        }
    }

    if result.state.running && !result.state.paused {
        core.events.state_changed(result.state.clone());
    } else if result.state_changed {
        core.events.state_changed(result.state.clone());
    }
    if result.completed {
        core.events.completed();
    }

    if !result.state.running || result.state.paused {
        return None;
    }

    let Some(started_at) = result.state.started_at else {
        return None;
    };
    let elapsed_millis = now_millis.saturating_sub(started_at).max(0);
    let remainder = elapsed_millis % SECOND_MILLIS;
    let delay_millis = if remainder == 0 {
        SECOND_MILLIS
    } else {
        SECOND_MILLIS - remainder
    };

    Some(Duration::from_millis(
        u64::try_from(delay_millis).unwrap_or(1_000),
    ))
}

fn flush_pending_saves(core: &RuntimeCore) {
    loop {
        let document = match core.pending_saves.lock() {
            Ok(mut pending) => pending.pop_front(),
            Err(_) => {
                eprintln!("[pomodoro] save_failed: persistence queue unavailable");
                return;
            }
        };
        let Some(document) = document else {
            return;
        };

        if let Err(error) = core.repository.save(&document) {
            eprintln!("[pomodoro] save_failed: {error}");
        }
    }
}

fn current_timestamp_millis() -> i64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    i64::try_from(millis).unwrap_or(MAXIMUM_SAFE_INTEGER)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicI64, Ordering};

    use super::*;

    #[derive(Debug, Default)]
    struct MemoryRepository {
        loaded: Mutex<PomodoroLoad>,
        saved: Mutex<Vec<PersistedPomodoroDocument>>,
    }

    impl MemoryRepository {
        fn with_load(load: PomodoroLoad) -> Self {
            Self {
                loaded: Mutex::new(load),
                saved: Mutex::new(Vec::new()),
            }
        }
    }

    impl Default for PomodoroLoad {
        fn default() -> Self {
            Self::Missing
        }
    }

    impl PomodoroRepository for MemoryRepository {
        fn load(&self) -> Result<PomodoroLoad, PomodoroRepositoryError> {
            Ok(self.loaded.lock().expect("memory load").clone())
        }

        fn save(
            &self,
            document: &PersistedPomodoroDocument,
        ) -> Result<(), PomodoroRepositoryError> {
            self.saved
                .lock()
                .expect("memory saves")
                .push(document.clone());
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    struct RecordedEvents {
        states: Mutex<Vec<PomodoroState>>,
        completions: Mutex<usize>,
    }

    impl PomodoroEvents for RecordedEvents {
        fn state_changed(&self, state: PomodoroState) {
            self.states.lock().expect("states").push(state);
        }

        fn completed(&self) {
            *self.completions.lock().expect("completions") += 1;
        }
    }

    fn running_state(remaining_seconds: u32, started_at: i64) -> PomodoroState {
        PomodoroState {
            running: true,
            paused: false,
            selected_duration_minutes: 25,
            duration_minutes: 25,
            remaining_seconds,
            started_at: Some(started_at),
        }
    }

    #[test]
    fn state_schema_matches_electron_and_rejects_invalid_combinations() {
        let idle = PomodoroState::default();
        assert_eq!(
            serde_json::to_value(&idle).expect("idle JSON"),
            serde_json::json!({
                "running": false,
                "paused": false,
                "selectedDurationMinutes": 25,
                "durationMinutes": 25,
                "remainingSeconds": 0,
                "startedAt": null
            })
        );
        assert_eq!(idle.validate(), Ok(()));

        let invalid = PomodoroState {
            running: false,
            paused: true,
            ..idle
        };
        assert_eq!(invalid.validate(), Err(PomodoroError::InvalidState));
    }

    #[test]
    fn engine_preserves_start_pause_resume_stop_and_duration_semantics() {
        let mut engine = PomodoroEngine::new(PomodoroState::default()).expect("engine");

        assert_eq!(
            engine.start(1, 1_000).expect("start").state,
            PomodoroState {
                running: true,
                paused: false,
                selected_duration_minutes: 1,
                duration_minutes: 1,
                remaining_seconds: 60,
                started_at: Some(1_000),
            }
        );
        assert_eq!(engine.snapshot(6_000).remaining_seconds, 55);

        let paused = engine.pause(6_000).state;
        assert!(paused.paused);
        assert_eq!(paused.remaining_seconds, 55);
        assert_eq!(engine.snapshot(36_000).remaining_seconds, 55);

        let resumed = engine.resume(36_000).state;
        assert!(!resumed.paused);
        assert_eq!(resumed.started_at, Some(36_000));
        assert_eq!(engine.snapshot(37_000).remaining_seconds, 54);

        let stopped = engine.stop().expect("stop").state;
        assert_eq!(stopped, PomodoroState::idle(1).expect("selected idle"));
    }

    #[test]
    fn elapsed_wall_clock_completes_exactly_once() {
        let mut engine = PomodoroEngine::new(running_state(30, 1_000)).expect("engine");

        let completed = engine.reconcile(60_000);
        assert!(completed.completed);
        assert!(completed.state_changed);
        assert!(!completed.state.running);

        let second = engine.reconcile(61_000);
        assert!(!second.completed);
        assert!(!second.state_changed);
    }

    #[test]
    fn selected_duration_remains_separate_from_an_active_session() {
        let mut engine = PomodoroEngine::new(PomodoroState::default()).expect("engine");
        engine.set_duration(45, 0).expect("select duration");
        engine.start(45, 0).expect("start");
        engine.set_duration(15, 5_000).expect("select next");

        let state = engine.snapshot(10_000);
        assert_eq!(state.duration_minutes, 45);
        assert_eq!(state.selected_duration_minutes, 15);
        assert_eq!(state.remaining_seconds, 2_690);
        assert_eq!(engine.start(0, 10_000), Err(PomodoroError::InvalidDuration));
        assert_eq!(
            engine.set_duration(721, 10_000),
            Err(PomodoroError::InvalidDuration)
        );
    }

    #[test]
    fn runtime_restores_and_materializes_a_running_document() {
        let repository = Arc::new(MemoryRepository::with_load(PomodoroLoad::Document(
            PersistedPomodoroDocument::new(running_state(1_500, 10_000)),
        )));
        let events = Arc::new(RecordedEvents::default());
        let clock = Arc::new(AtomicI64::new(70_000));
        let runtime = PomodoroRuntime::with_clock(repository, events, {
            let clock = Arc::clone(&clock);
            Arc::new(move || clock.load(Ordering::SeqCst))
        });

        let restored = runtime.start().expect("runtime start");
        assert_eq!(restored.remaining_seconds, 1_440);
        runtime.stop().expect("runtime stop");
    }

    #[test]
    fn runtime_completes_an_expired_restored_session_once() {
        let repository = Arc::new(MemoryRepository::with_load(PomodoroLoad::Document(
            PersistedPomodoroDocument::new(running_state(30, 1_000)),
        )));
        let events = Arc::new(RecordedEvents::default());
        let clock = Arc::new(AtomicI64::new(60_000));
        let runtime = PomodoroRuntime::with_clock(
            Arc::clone(&repository) as Arc<dyn PomodoroRepository>,
            Arc::clone(&events) as Arc<dyn PomodoroEvents>,
            {
                let clock = Arc::clone(&clock);
                Arc::new(move || clock.load(Ordering::SeqCst))
            },
        );

        let restored = runtime.start().expect("runtime start");
        assert!(!restored.running);
        assert_eq!(*events.completions.lock().expect("completions"), 1);
        assert_eq!(repository.saved.lock().expect("saves").len(), 1);

        assert!(!runtime.state().expect("second state").running);
        assert_eq!(*events.completions.lock().expect("completions"), 1);
        runtime.stop().expect("runtime stop");
    }

    #[test]
    fn runtime_starts_only_one_scheduler_and_flushes_queued_saves_on_stop() {
        let repository = Arc::new(MemoryRepository::default());
        let events = Arc::new(RecordedEvents::default());
        let runtime = PomodoroRuntime::new(
            Arc::clone(&repository) as Arc<dyn PomodoroRepository>,
            events,
        );

        runtime.start().expect("first start");
        runtime.start().expect("second start");
        runtime.start_session(1).expect("session start");
        runtime.pause().expect("pause");
        runtime.resume().expect("resume");
        runtime.stop_session().expect("session stop");
        runtime.stop().expect("runtime stop");

        let saves = repository.saved.lock().expect("saves");
        assert!(saves.len() >= 5);
        assert!(!saves.last().expect("last save").state.running);
    }

    #[test]
    fn pending_event_queue_retains_latest_state_and_one_completion() {
        let events = PomodoroEventQueue::default();
        let state = running_state(60, 1_000);

        events.state_changed(state.clone());
        events.completed();
        events.completed();

        assert_eq!(events.latest_state().expect("latest state"), Some(state));
        assert!(events.has_pending_completion().expect("pending completion"));
    }
}
