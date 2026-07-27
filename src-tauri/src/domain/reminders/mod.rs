use std::{
    cmp::Ordering,
    collections::{HashSet, VecDeque},
    sync::{Arc, Condvar, Mutex, PoisonError},
    thread::{self, JoinHandle},
    time::Duration,
};

use chrono::{
    DateTime, Datelike, Days, Local, LocalResult, Months, NaiveDateTime, TimeZone, Timelike, Utc,
};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use uuid::Uuid;

pub(crate) const CLOCK_VALIDATION_INTERVAL: Duration = Duration::from_secs(60);
pub(crate) const MAXIMUM_OVERDUE_AGE: Duration = Duration::from_secs(24 * 60 * 60);
pub(crate) const MAXIMUM_REMINDER_TITLE_LENGTH: usize = 60;
pub(crate) const MAXIMUM_REMINDER_MESSAGE_LENGTH: usize = 250;
pub(crate) const MAXIMUM_REMINDER_ID_LENGTH: usize = 128;
pub(crate) const MINIMUM_REMINDER_INTERVAL_VALUE: u32 = 1;
pub(crate) const MAXIMUM_REMINDER_INTERVAL_VALUE: u32 = 100_000;

const MILLISECONDS_PER_MINUTE: i64 = 60_000;
const MILLISECONDS_PER_HOUR: i64 = 60 * MILLISECONDS_PER_MINUTE;
const MILLISECONDS_PER_DAY: i64 = 24 * MILLISECONDS_PER_HOUR;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ReminderIntervalUnit {
    Minutes,
    Hours,
    Days,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum ReminderRecurrence {
    None,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Interval {
        unit: ReminderIntervalUnit,
        value: u32,
    },
}

impl Default for ReminderRecurrence {
    fn default() -> Self {
        Self::None
    }
}

impl ReminderRecurrence {
    fn validate(&self) -> Result<(), ReminderValidationError> {
        if let Self::Interval { value, .. } = self {
            if !(MINIMUM_REMINDER_INTERVAL_VALUE..=MAXIMUM_REMINDER_INTERVAL_VALUE).contains(value)
            {
                return Err(ReminderValidationError::new(
                    ReminderValidationField::Recurrence,
                    "Reminder recurrence is invalid.",
                ));
            }
        }

        Ok(())
    }

    fn is_recurring(&self) -> bool {
        !matches!(self, Self::None)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Reminder {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) message: String,
    pub(crate) scheduled_at: String,
    pub(crate) recurrence: ReminderRecurrence,
    pub(crate) last_triggered_at: Option<String>,
    pub(crate) next_occurrence: Option<String>,
    pub(crate) completed: bool,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredReminder {
    id: String,
    title: String,
    message: String,
    scheduled_at: String,
    #[serde(default)]
    recurrence: OptionalStoredField<ReminderRecurrence>,
    #[serde(default)]
    last_triggered_at: OptionalStoredField<String>,
    #[serde(default)]
    next_occurrence: OptionalStoredField<String>,
    completed: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Default)]
enum OptionalStoredField<T> {
    #[default]
    Missing,
    Null,
    Value(T),
}

impl<'de, T> Deserialize<'de> for OptionalStoredField<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<T>::deserialize(deserializer).map(|value| value.map_or(Self::Null, Self::Value))
    }
}

impl<'de> Deserialize<'de> for Reminder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let stored = StoredReminder::deserialize(deserializer)?;
        let recurrence = match stored.recurrence {
            OptionalStoredField::Missing => ReminderRecurrence::None,
            OptionalStoredField::Value(value) => value,
            OptionalStoredField::Null => {
                return Err(D::Error::custom("Reminder recurrence is invalid."));
            }
        };
        let last_triggered_at = match stored.last_triggered_at {
            OptionalStoredField::Missing | OptionalStoredField::Null => None,
            OptionalStoredField::Value(value) => Some(value),
        };
        let next_occurrence = match stored.next_occurrence {
            OptionalStoredField::Missing if stored.completed => None,
            OptionalStoredField::Missing => Some(stored.scheduled_at.clone()),
            OptionalStoredField::Null => None,
            OptionalStoredField::Value(value) => Some(value),
        };
        let title = stored.title.clone();
        let message = stored.message.clone();
        let mut reminder = Self {
            id: stored.id,
            title: stored.title,
            message: stored.message,
            scheduled_at: stored.scheduled_at,
            recurrence,
            last_triggered_at,
            next_occurrence,
            completed: stored.completed,
            created_at: stored.created_at,
            updated_at: stored.updated_at,
        };

        reminder
            .validate_and_canonicalize()
            .map_err(D::Error::custom)?;

        if reminder.title != title || reminder.message != message {
            return Err(D::Error::custom(
                "Stored reminder text must already be normalized.",
            ));
        }

        Ok(reminder)
    }
}

impl Reminder {
    pub(crate) fn validate_and_canonicalize(&mut self) -> Result<(), ReminderValidationError> {
        self.id = normalize_id(&self.id)?;
        self.title = normalize_title(&self.title)?;
        self.message = normalize_message(Some(&self.message))?;
        self.scheduled_at = canonical_timestamp(&self.scheduled_at, "scheduledAt")?;
        self.recurrence.validate()?;
        self.last_triggered_at = self
            .last_triggered_at
            .as_deref()
            .map(|value| canonical_timestamp(value, "lastTriggeredAt"))
            .transpose()?;
        self.next_occurrence = self
            .next_occurrence
            .as_deref()
            .map(|value| canonical_timestamp(value, "nextOccurrence"))
            .transpose()?;
        self.created_at = canonical_timestamp(&self.created_at, "createdAt")?;
        self.updated_at = canonical_timestamp(&self.updated_at, "updatedAt")?;

        if timestamp_millis(&self.updated_at, "updatedAt")?
            < timestamp_millis(&self.created_at, "createdAt")?
        {
            return Err(ReminderValidationError::new(
                ReminderValidationField::Reminder,
                "Reminder timestamps are invalid.",
            ));
        }

        if (!self.completed && self.next_occurrence.is_none())
            || (self.completed
                && (self.recurrence.is_recurring() || self.next_occurrence.is_some()))
        {
            return Err(ReminderValidationError::new(
                ReminderValidationField::Reminder,
                "Reminder state is invalid.",
            ));
        }

        Ok(())
    }

    fn schedule_millis(&self) -> Result<i64, ReminderValidationError> {
        timestamp_millis(
            self.next_occurrence
                .as_deref()
                .unwrap_or(&self.scheduled_at),
            "nextOccurrence",
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CreateReminderInput {
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) message: Option<String>,
    pub(crate) scheduled_at: String,
    #[serde(default)]
    pub(crate) recurrence: Option<ReminderRecurrence>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateReminderInput {
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) message: Option<String>,
    #[serde(default)]
    pub(crate) scheduled_at: Option<String>,
    #[serde(default)]
    pub(crate) recurrence: Option<ReminderRecurrence>,
}

impl UpdateReminderInput {
    fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.message.is_none()
            && self.scheduled_at.is_none()
            && self.recurrence.is_none()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ReminderFiredNotification {
    pub(crate) reminder: Reminder,
    pub(crate) fired_at: String,
    pub(crate) overdue: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReminderValidationField {
    Id,
    Title,
    Message,
    ScheduledAt,
    Recurrence,
    Reminder,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReminderValidationError {
    pub(crate) field: ReminderValidationField,
    message: String,
}

impl ReminderValidationError {
    fn new(field: ReminderValidationField, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ReminderValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ReminderValidationError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReminderRepositoryError {
    message: String,
}

impl ReminderRepositoryError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ReminderRepositoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ReminderRepositoryError {}

pub(crate) trait ReminderRepository: Send + Sync {
    fn list(&self) -> Result<Vec<Reminder>, ReminderRepositoryError>;
    fn save(&self, reminders: &[Reminder]) -> Result<(), ReminderRepositoryError>;
}

#[derive(Debug)]
pub(crate) enum ReminderServiceError {
    Validation(ReminderValidationError),
    NotFound,
    Repository(ReminderRepositoryError),
    StateUnavailable,
    ClockUnavailable,
    IdUnavailable,
    RecurrenceUnavailable,
}

impl std::fmt::Display for ReminderServiceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(error) => error.fmt(formatter),
            Self::NotFound => formatter.write_str("Reminder not found."),
            Self::Repository(error) => error.fmt(formatter),
            Self::StateUnavailable => formatter.write_str("Reminder state is unavailable."),
            Self::ClockUnavailable => formatter.write_str("Reminder clock is unavailable."),
            Self::IdUnavailable => formatter.write_str("Unable to create a unique reminder ID."),
            Self::RecurrenceUnavailable => {
                formatter.write_str("Unable to calculate the next reminder occurrence.")
            }
        }
    }
}

impl std::error::Error for ReminderServiceError {}

impl From<ReminderValidationError> for ReminderServiceError {
    fn from(error: ReminderValidationError) -> Self {
        Self::Validation(error)
    }
}

impl From<ReminderRepositoryError> for ReminderServiceError {
    fn from(error: ReminderRepositoryError) -> Self {
        Self::Repository(error)
    }
}

impl<T> From<PoisonError<T>> for ReminderServiceError {
    fn from(_error: PoisonError<T>) -> Self {
        Self::StateUnavailable
    }
}

type Clock = Arc<dyn Fn() -> i64 + Send + Sync>;
type IdGenerator = Arc<dyn Fn() -> String + Send + Sync>;
pub(crate) type DeliveryEmitter =
    Arc<dyn Fn(ReminderFiredNotification) -> Result<(), String> + Send + Sync>;

pub(crate) struct ReminderService {
    repository: Arc<dyn ReminderRepository>,
    mutation_lock: Mutex<()>,
    wake: Arc<SchedulerWake>,
    now: Clock,
    create_id: IdGenerator,
}

impl std::fmt::Debug for ReminderService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReminderService")
            .field("repository", &"<repository>")
            .finish_non_exhaustive()
    }
}

impl ReminderService {
    pub(crate) fn new(repository: Arc<dyn ReminderRepository>) -> Self {
        Self::with_dependencies(
            repository,
            Arc::new(current_timestamp_millis),
            Arc::new(|| Uuid::new_v4().to_string()),
            Arc::new(SchedulerWake::default()),
        )
    }

    fn with_dependencies(
        repository: Arc<dyn ReminderRepository>,
        now: Clock,
        create_id: IdGenerator,
        wake: Arc<SchedulerWake>,
    ) -> Self {
        Self {
            repository,
            mutation_lock: Mutex::new(()),
            wake,
            now,
            create_id,
        }
    }

    pub(crate) fn create(
        &self,
        input: CreateReminderInput,
    ) -> Result<Reminder, ReminderServiceError> {
        let _guard = self.mutation_lock.lock()?;
        let now = self.current_timestamp()?;
        let title = normalize_title(&input.title)?;
        let message = normalize_message(input.message.as_deref())?;
        let scheduled_at = canonical_timestamp(&input.scheduled_at, "scheduledAt")?;

        if timestamp_millis(&scheduled_at, "scheduledAt")? < now {
            return Err(ReminderValidationError::new(
                ReminderValidationField::ScheduledAt,
                "Reminder date must not be in the past.",
            )
            .into());
        }

        let recurrence = input.recurrence.unwrap_or_default();
        recurrence.validate()?;
        let mut reminders = self.repository.list()?;
        canonicalize_and_sort(&mut reminders)?;
        let id = self.generate_unique_id(&reminders)?;
        let timestamp = format_timestamp(now).ok_or(ReminderServiceError::ClockUnavailable)?;
        let reminder = Reminder {
            id,
            title,
            message,
            scheduled_at: scheduled_at.clone(),
            recurrence,
            last_triggered_at: None,
            next_occurrence: Some(scheduled_at),
            completed: false,
            created_at: timestamp.clone(),
            updated_at: timestamp,
        };

        reminders.push(reminder.clone());
        self.persist(&mut reminders)?;
        Ok(reminder)
    }

    pub(crate) fn update(
        &self,
        id: &str,
        input: UpdateReminderInput,
    ) -> Result<Reminder, ReminderServiceError> {
        let _guard = self.mutation_lock.lock()?;
        let id = normalize_id(id)?;

        if input.is_empty() {
            return Err(ReminderValidationError::new(
                ReminderValidationField::Reminder,
                "Invalid reminder update.",
            )
            .into());
        }

        let now = self.current_timestamp()?;
        let mut reminders = self.repository.list()?;
        canonicalize_and_sort(&mut reminders)?;
        let index = reminders
            .iter()
            .position(|reminder| reminder.id == id)
            .ok_or(ReminderServiceError::NotFound)?;
        let current = reminders[index].clone();
        let title = input
            .title
            .as_deref()
            .map(normalize_title)
            .transpose()?
            .unwrap_or_else(|| current.title.clone());
        let message = input
            .message
            .as_deref()
            .map(|value| normalize_message(Some(value)))
            .transpose()?
            .unwrap_or_else(|| current.message.clone());
        let scheduled_at = input
            .scheduled_at
            .as_deref()
            .map(|value| canonical_timestamp(value, "scheduledAt"))
            .transpose()?
            .unwrap_or_else(|| current.scheduled_at.clone());

        if input.scheduled_at.is_some() && timestamp_millis(&scheduled_at, "scheduledAt")? < now {
            return Err(ReminderValidationError::new(
                ReminderValidationField::ScheduledAt,
                "Reminder date must not be in the past.",
            )
            .into());
        }

        let recurrence = input
            .recurrence
            .clone()
            .unwrap_or_else(|| current.recurrence.clone());
        recurrence.validate()?;
        let schedule_changed = input.scheduled_at.is_some();
        let recurrence_changed = input
            .recurrence
            .as_ref()
            .is_some_and(|value| value != &current.recurrence);
        let mut completed = current.completed;
        let mut last_triggered_at = current.last_triggered_at.clone();
        let mut next_occurrence = current.next_occurrence.clone();

        if schedule_changed {
            completed = false;
            last_triggered_at = None;
            next_occurrence = Some(scheduled_at.clone());
        }

        if recurrence_changed {
            last_triggered_at = None;

            if recurrence.is_recurring() {
                completed = false;

                if next_occurrence
                    .as_deref()
                    .map(|value| timestamp_millis(value, "nextOccurrence"))
                    .transpose()?
                    .map_or(true, |value| value <= now)
                {
                    next_occurrence = Some(calculate_next_occurrence(
                        &recurrence,
                        &scheduled_at,
                        now,
                        &scheduled_at,
                    )?);
                }
            }
        }

        let updated_at = format_timestamp(now).ok_or(ReminderServiceError::ClockUnavailable)?;
        let reminder = Reminder {
            id: current.id,
            title,
            message,
            scheduled_at,
            recurrence,
            last_triggered_at,
            next_occurrence,
            completed,
            created_at: current.created_at,
            updated_at,
        };

        reminders[index] = reminder.clone();
        self.persist(&mut reminders)?;
        Ok(reminder)
    }

    pub(crate) fn delete(&self, id: &str) -> Result<bool, ReminderServiceError> {
        let _guard = self.mutation_lock.lock()?;
        let id = normalize_id(id)?;
        let mut reminders = self.repository.list()?;
        canonicalize_and_sort(&mut reminders)?;
        let original_length = reminders.len();
        reminders.retain(|reminder| reminder.id != id);

        if reminders.len() == original_length {
            return Ok(false);
        }

        self.persist(&mut reminders)?;
        Ok(true)
    }

    pub(crate) fn get(&self, id: &str) -> Result<Option<Reminder>, ReminderServiceError> {
        let id = normalize_id(id)?;
        let mut reminders = self.repository.list()?;
        canonicalize_and_sort(&mut reminders)?;
        Ok(reminders.into_iter().find(|reminder| reminder.id == id))
    }

    pub(crate) fn list(&self) -> Result<Vec<Reminder>, ReminderServiceError> {
        let mut reminders = self.repository.list()?;
        canonicalize_and_sort(&mut reminders)?;
        Ok(reminders)
    }

    pub(crate) fn mark_completed(&self, id: &str) -> Result<Reminder, ReminderServiceError> {
        let _guard = self.mutation_lock.lock()?;
        let id = normalize_id(id)?;
        let mut reminders = self.repository.list()?;
        canonicalize_and_sort(&mut reminders)?;
        let index = reminders
            .iter()
            .position(|reminder| reminder.id == id)
            .ok_or(ReminderServiceError::NotFound)?;
        let current = reminders[index].clone();

        if current.completed {
            return Ok(current);
        }

        let now = self.current_timestamp()?;
        let timestamp = format_timestamp(now).ok_or(ReminderServiceError::ClockUnavailable)?;
        let reminder = if current.recurrence.is_recurring() {
            Reminder {
                next_occurrence: Some(calculate_next_occurrence(
                    &current.recurrence,
                    current
                        .next_occurrence
                        .as_deref()
                        .unwrap_or(&current.scheduled_at),
                    now,
                    &current.scheduled_at,
                )?),
                last_triggered_at: Some(timestamp.clone()),
                completed: false,
                updated_at: timestamp,
                ..current
            }
        } else {
            Reminder {
                recurrence: ReminderRecurrence::None,
                last_triggered_at: Some(timestamp.clone()),
                next_occurrence: None,
                completed: true,
                updated_at: timestamp,
                ..current
            }
        };

        reminders[index] = reminder.clone();
        self.persist(&mut reminders)?;
        Ok(reminder)
    }

    fn current_timestamp(&self) -> Result<i64, ReminderServiceError> {
        let value = (self.now)();
        format_timestamp(value)
            .map(|_| value)
            .ok_or(ReminderServiceError::ClockUnavailable)
    }

    fn generate_unique_id(&self, reminders: &[Reminder]) -> Result<String, ReminderServiceError> {
        let existing = reminders
            .iter()
            .map(|reminder| reminder.id.as_str())
            .collect::<HashSet<_>>();

        for _ in 0..10 {
            let candidate = normalize_id(&(self.create_id)())?;

            if !existing.contains(candidate.as_str()) {
                return Ok(candidate);
            }
        }

        Err(ReminderServiceError::IdUnavailable)
    }

    fn persist(&self, reminders: &mut Vec<Reminder>) -> Result<(), ReminderServiceError> {
        canonicalize_and_sort(reminders)?;
        self.repository.save(reminders)?;
        self.wake.notify();
        Ok(())
    }
}

pub(crate) trait ReminderSchedulerSource: Send + Sync {
    fn list_reminders(&self) -> Result<Vec<Reminder>, ReminderServiceError>;
    fn mark_reminder_completed(&self, id: &str) -> Result<Reminder, ReminderServiceError>;
}

impl ReminderSchedulerSource for ReminderService {
    fn list_reminders(&self) -> Result<Vec<Reminder>, ReminderServiceError> {
        self.list()
    }

    fn mark_reminder_completed(&self, id: &str) -> Result<Reminder, ReminderServiceError> {
        self.mark_completed(id)
    }
}

pub(crate) trait ReminderEventSink: Send + Sync {
    fn emit(&self, notification: ReminderFiredNotification);
}

#[derive(Default)]
pub(crate) struct PendingReminderDeliveries {
    state: Mutex<ReminderDeliveryState>,
    emitter: Option<DeliveryEmitter>,
}

impl std::fmt::Debug for PendingReminderDeliveries {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingReminderDeliveries")
            .field("state", &self.state)
            .field("emitter_configured", &self.emitter.is_some())
            .finish()
    }
}

impl PendingReminderDeliveries {
    fn with_emitter(emitter: DeliveryEmitter) -> Self {
        Self {
            state: Mutex::new(ReminderDeliveryState::default()),
            emitter: Some(emitter),
        }
    }

    pub(crate) fn activate(&self) -> Result<(), ReminderSchedulerError> {
        let generation = {
            let mut state = self.state.lock()?;
            state.generation = state.generation.wrapping_add(1);
            state.active = false;
            state.generation
        };

        self.flush(generation);
        Ok(())
    }

    pub(crate) fn deactivate(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.generation = state.generation.wrapping_add(1);
            state.active = false;
        }
    }

    fn enqueue(&self, notification: ReminderFiredNotification) {
        let flush_generation = match self.state.lock() {
            Ok(mut state) => {
                state.notifications.push_back(notification);

                if !state.active {
                    None
                } else {
                    state.active = false;
                    Some(state.generation)
                }
            }
            Err(_) => {
                eprintln!("[reminder-scheduler] delivery_queue_unavailable");
                None
            }
        };

        if let Some(generation) = flush_generation {
            self.flush(generation);
        }
    }

    fn flush(&self, generation: u64) {
        let Some(emitter) = self.emitter.as_ref() else {
            return;
        };

        loop {
            let notification = match self.state.lock() {
                Ok(mut state) if state.generation == generation => {
                    match state.notifications.pop_front() {
                        Some(notification) => notification,
                        None => {
                            state.active = true;
                            return;
                        }
                    }
                }
                Ok(_) => return,
                Err(_) => {
                    eprintln!("[reminder-scheduler] delivery_queue_unavailable");
                    return;
                }
            };

            if let Err(error) = emitter(notification.clone()) {
                eprintln!("[reminder-scheduler] delivery_failed: {error}");

                if let Ok(mut state) = self.state.lock() {
                    if state.generation == generation {
                        state.notifications.push_front(notification);
                        state.active = false;
                    }
                }
                return;
            }
        }
    }
}

impl ReminderEventSink for PendingReminderDeliveries {
    fn emit(&self, notification: ReminderFiredNotification) {
        self.enqueue(notification);
    }
}

#[derive(Debug, Default)]
struct ReminderDeliveryState {
    active: bool,
    generation: u64,
    notifications: VecDeque<ReminderFiredNotification>,
}

#[derive(Debug, Default)]
struct SchedulerWake {
    state: Mutex<SchedulerWakeState>,
    changed: Condvar,
}

#[derive(Debug, Default)]
struct SchedulerWakeState {
    generation: u64,
    stopped: bool,
}

impl SchedulerWake {
    fn notify(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.generation = state.generation.wrapping_add(1);
            self.changed.notify_all();
        }
    }

    fn reset(&self) -> Result<(), ReminderSchedulerError> {
        let mut state = self.state.lock().map_err(ReminderSchedulerError::from)?;
        state.stopped = false;
        state.generation = state.generation.wrapping_add(1);
        self.changed.notify_all();
        Ok(())
    }

    fn stop(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.stopped = true;
            state.generation = state.generation.wrapping_add(1);
            self.changed.notify_all();
        }
    }

    fn wait(&self, delay: Option<Duration>) -> Result<bool, ReminderSchedulerError> {
        let state = self.state.lock().map_err(ReminderSchedulerError::from)?;
        let generation = state.generation;

        if state.stopped {
            return Ok(false);
        }

        let state = match delay {
            Some(delay) => {
                let (state, _) = self
                    .changed
                    .wait_timeout_while(state, delay, |state| {
                        !state.stopped && state.generation == generation
                    })
                    .map_err(ReminderSchedulerError::from)?;
                state
            }
            None => self
                .changed
                .wait_while(state, |state| {
                    !state.stopped && state.generation == generation
                })
                .map_err(ReminderSchedulerError::from)?,
        };

        Ok(!state.stopped)
    }
}

#[derive(Debug)]
pub(crate) enum ReminderSchedulerError {
    StateUnavailable,
    ThreadUnavailable,
}

impl<T> From<PoisonError<T>> for ReminderSchedulerError {
    fn from(_error: PoisonError<T>) -> Self {
        Self::StateUnavailable
    }
}

impl std::fmt::Display for ReminderSchedulerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StateUnavailable => {
                formatter.write_str("Reminder scheduler state is unavailable.")
            }
            Self::ThreadUnavailable => {
                formatter.write_str("Reminder scheduler thread could not be started.")
            }
        }
    }
}

impl std::error::Error for ReminderSchedulerError {}

pub(crate) struct ReminderScheduler {
    source: Arc<dyn ReminderSchedulerSource>,
    sink: Arc<dyn ReminderEventSink>,
    wake: Arc<SchedulerWake>,
    now: Clock,
    thread: Mutex<Option<JoinHandle<()>>>,
    clock_validation_interval: Duration,
    maximum_overdue_age: Duration,
}

impl std::fmt::Debug for ReminderScheduler {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReminderScheduler")
            .field("running", &self.is_running())
            .finish_non_exhaustive()
    }
}

impl ReminderScheduler {
    pub(crate) fn new(service: Arc<ReminderService>, sink: Arc<dyn ReminderEventSink>) -> Self {
        Self::with_dependencies(
            service.clone(),
            sink,
            Arc::clone(&service.wake),
            Arc::new(current_timestamp_millis),
            CLOCK_VALIDATION_INTERVAL,
            MAXIMUM_OVERDUE_AGE,
        )
    }

    fn with_dependencies(
        source: Arc<dyn ReminderSchedulerSource>,
        sink: Arc<dyn ReminderEventSink>,
        wake: Arc<SchedulerWake>,
        now: Clock,
        clock_validation_interval: Duration,
        maximum_overdue_age: Duration,
    ) -> Self {
        Self {
            source,
            sink,
            wake,
            now,
            thread: Mutex::new(None),
            clock_validation_interval,
            maximum_overdue_age,
        }
    }

    pub(crate) fn start(&self) -> Result<(), ReminderSchedulerError> {
        let mut thread_slot = self.thread.lock()?;

        if thread_slot.is_some() {
            return Ok(());
        }

        self.wake.reset()?;
        let source = Arc::clone(&self.source);
        let sink = Arc::clone(&self.sink);
        let wake = Arc::clone(&self.wake);
        let now = Arc::clone(&self.now);
        let clock_validation_interval = self.clock_validation_interval;
        let maximum_overdue_age = self.maximum_overdue_age;
        let thread = thread::Builder::new()
            .name("ducky-reminder-scheduler".to_owned())
            .spawn(move || {
                let mut state = SchedulerState::default();

                loop {
                    let delay = state.synchronize(
                        source.as_ref(),
                        sink.as_ref(),
                        now(),
                        clock_validation_interval,
                        maximum_overdue_age,
                    );

                    if !wake.wait(delay).unwrap_or(false) {
                        break;
                    }
                }
            })
            .map_err(|_| ReminderSchedulerError::ThreadUnavailable)?;

        *thread_slot = Some(thread);
        Ok(())
    }

    pub(crate) fn stop(&self) -> Result<(), ReminderSchedulerError> {
        self.wake.stop();
        let thread = self.thread.lock()?.take();

        if let Some(thread) = thread {
            thread
                .join()
                .map_err(|_| ReminderSchedulerError::ThreadUnavailable)?;
        }

        Ok(())
    }

    pub(crate) fn resynchronize(&self) {
        self.wake.notify();
    }

    pub(crate) fn is_running(&self) -> bool {
        self.thread
            .lock()
            .map(|thread| thread.is_some())
            .unwrap_or(false)
    }
}

impl Drop for ReminderScheduler {
    fn drop(&mut self) {
        self.wake.stop();

        if let Ok(thread) = self.thread.get_mut() {
            if let Some(thread) = thread.take() {
                let _ = thread.join();
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct ReminderRuntime {
    pub(crate) service: Arc<ReminderService>,
    pub(crate) pending_deliveries: Arc<PendingReminderDeliveries>,
    scheduler: ReminderScheduler,
}

impl ReminderRuntime {
    pub(crate) fn with_delivery(
        repository: Arc<dyn ReminderRepository>,
        emitter: DeliveryEmitter,
    ) -> Self {
        Self::build(repository, Some(emitter))
    }

    fn build(repository: Arc<dyn ReminderRepository>, emitter: Option<DeliveryEmitter>) -> Self {
        let service = Arc::new(ReminderService::new(repository));
        let pending_deliveries = Arc::new(emitter.map_or_else(
            PendingReminderDeliveries::default,
            PendingReminderDeliveries::with_emitter,
        ));
        let scheduler = ReminderScheduler::new(service.clone(), pending_deliveries.clone());

        Self {
            service,
            pending_deliveries,
            scheduler,
        }
    }

    pub(crate) fn start(&self) -> Result<(), ReminderSchedulerError> {
        self.scheduler.start()
    }

    pub(crate) fn stop(&self) -> Result<(), ReminderSchedulerError> {
        self.scheduler.stop()
    }

    pub(crate) fn resynchronize(&self) {
        self.scheduler.resynchronize();
    }
}

#[derive(Debug, Default)]
struct SchedulerState {
    fired_reminder_ids: HashSet<String>,
}

impl SchedulerState {
    fn synchronize(
        &mut self,
        source: &dyn ReminderSchedulerSource,
        sink: &dyn ReminderEventSink,
        now: i64,
        clock_validation_interval: Duration,
        maximum_overdue_age: Duration,
    ) -> Option<Duration> {
        let mut candidates = match source.list_reminders() {
            Ok(reminders) => reminders
                .into_iter()
                .filter_map(|reminder| {
                    if reminder.completed {
                        return None;
                    }

                    reminder
                        .next_occurrence
                        .as_deref()
                        .and_then(|value| timestamp_millis(value, "nextOccurrence").ok())
                        .map(|timestamp| (reminder, timestamp))
                })
                .collect::<Vec<_>>(),
            Err(error) => {
                eprintln!("[reminder-scheduler] synchronization_failed: {error}");
                return Some(clock_validation_interval);
            }
        };
        candidates.sort_by(|(left, left_timestamp), (right, right_timestamp)| {
            left_timestamp
                .cmp(right_timestamp)
                .then_with(|| left.id.cmp(&right.id))
        });
        let candidate_ids = candidates
            .iter()
            .map(|(reminder, _)| reminder.id.as_str())
            .collect::<HashSet<_>>();
        self.fired_reminder_ids
            .retain(|id| candidate_ids.contains(id.as_str()));
        let maximum_overdue_millis =
            i64::try_from(maximum_overdue_age.as_millis()).unwrap_or(i64::MAX);

        for (reminder, timestamp) in candidates.iter().filter(|(_, timestamp)| {
            *timestamp <= now && now - *timestamp <= maximum_overdue_millis
        }) {
            if self.fired_reminder_ids.insert(reminder.id.clone()) {
                sink.emit(ReminderFiredNotification {
                    reminder: reminder.clone(),
                    fired_at: format_timestamp(now).unwrap_or_else(|| reminder.updated_at.clone()),
                    overdue: *timestamp < now,
                });
            }

            match source.mark_reminder_completed(&reminder.id) {
                Ok(_) => {
                    self.fired_reminder_ids.remove(&reminder.id);
                }
                Err(error) => {
                    eprintln!(
                        "[reminder-scheduler] completion_failed: {}: {error}",
                        reminder.id
                    );
                }
            }
        }

        let refreshed = match source.list_reminders() {
            Ok(reminders) => reminders,
            Err(error) => {
                eprintln!("[reminder-scheduler] synchronization_failed: {error}");
                return Some(clock_validation_interval);
            }
        };
        let mut pending = refreshed
            .into_iter()
            .filter_map(|reminder| {
                if reminder.completed {
                    return None;
                }

                reminder
                    .next_occurrence
                    .as_deref()
                    .and_then(|value| timestamp_millis(value, "nextOccurrence").ok())
                    .map(|timestamp| (reminder, timestamp))
            })
            .collect::<Vec<_>>();
        pending.sort_by(|(left, left_timestamp), (right, right_timestamp)| {
            left_timestamp
                .cmp(right_timestamp)
                .then_with(|| left.id.cmp(&right.id))
        });
        let pending_due = pending
            .iter()
            .any(|(_, timestamp)| *timestamp <= now && now - *timestamp <= maximum_overdue_millis);

        if pending_due {
            return Some(clock_validation_interval);
        }

        if let Some((_, timestamp)) = pending.iter().find(|(_, timestamp)| *timestamp > now) {
            let delay = u64::try_from(*timestamp - now).unwrap_or(u64::MAX);
            return Some(clock_validation_interval.min(Duration::from_millis(delay)));
        }

        (!pending.is_empty()).then_some(clock_validation_interval)
    }
}

fn current_timestamp_millis() -> i64 {
    Utc::now().timestamp_millis()
}

fn format_timestamp(timestamp: i64) -> Option<String> {
    DateTime::<Utc>::from_timestamp_millis(timestamp)
        .map(|value| value.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}

fn canonical_timestamp(value: &str, field: &str) -> Result<String, ReminderValidationError> {
    let parsed = DateTime::parse_from_rfc3339(value).map_err(|_| {
        ReminderValidationError::new(
            timestamp_field(field),
            format!("Reminder {field} must be a valid ISO-8601 datetime."),
        )
    })?;
    Ok(parsed
        .with_timezone(&Utc)
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}

fn timestamp_millis(value: &str, field: &str) -> Result<i64, ReminderValidationError> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.timestamp_millis())
        .map_err(|_| {
            ReminderValidationError::new(
                timestamp_field(field),
                format!("Reminder {field} must be a valid ISO-8601 datetime."),
            )
        })
}

fn timestamp_field(field: &str) -> ReminderValidationField {
    if field == "scheduledAt" {
        ReminderValidationField::ScheduledAt
    } else {
        ReminderValidationField::Reminder
    }
}

fn normalize_id(value: &str) -> Result<String, ReminderValidationError> {
    if value.is_empty()
        || value.trim() != value
        || value.encode_utf16().count() > MAXIMUM_REMINDER_ID_LENGTH
    {
        return Err(ReminderValidationError::new(
            ReminderValidationField::Id,
            "Invalid reminder ID.",
        ));
    }

    Ok(value.to_owned())
}

fn normalize_title(value: &str) -> Result<String, ReminderValidationError> {
    let normalized = value.trim();

    if normalized.is_empty() {
        return Err(ReminderValidationError::new(
            ReminderValidationField::Title,
            "Reminder title is required.",
        ));
    }

    if normalized.encode_utf16().count() > MAXIMUM_REMINDER_TITLE_LENGTH {
        return Err(ReminderValidationError::new(
            ReminderValidationField::Title,
            format!("Reminder title must not exceed {MAXIMUM_REMINDER_TITLE_LENGTH} characters."),
        ));
    }

    Ok(normalized.to_owned())
}

fn normalize_message(value: Option<&str>) -> Result<String, ReminderValidationError> {
    let normalized = value.unwrap_or_default().trim();

    if normalized.encode_utf16().count() > MAXIMUM_REMINDER_MESSAGE_LENGTH {
        return Err(ReminderValidationError::new(
            ReminderValidationField::Message,
            format!(
                "Reminder message must not exceed {MAXIMUM_REMINDER_MESSAGE_LENGTH} characters."
            ),
        ));
    }

    Ok(normalized.to_owned())
}

fn canonicalize_and_sort(reminders: &mut Vec<Reminder>) -> Result<(), ReminderValidationError> {
    let mut ids = HashSet::new();

    for reminder in reminders.iter_mut() {
        reminder.validate_and_canonicalize()?;

        if !ids.insert(reminder.id.clone()) {
            return Err(ReminderValidationError::new(
                ReminderValidationField::Reminder,
                "Reminder IDs must be unique.",
            ));
        }
    }

    reminders.sort_by(compare_reminders);
    Ok(())
}

fn compare_reminders(left: &Reminder, right: &Reminder) -> Ordering {
    left.schedule_millis()
        .unwrap_or(i64::MAX)
        .cmp(&right.schedule_millis().unwrap_or(i64::MAX))
        .then_with(|| {
            timestamp_millis(&left.created_at, "createdAt")
                .unwrap_or(i64::MAX)
                .cmp(&timestamp_millis(&right.created_at, "createdAt").unwrap_or(i64::MAX))
        })
        .then_with(|| left.id.cmp(&right.id))
}

fn calculate_next_occurrence(
    recurrence: &ReminderRecurrence,
    current_occurrence: &str,
    after_timestamp: i64,
    anchor_scheduled_at: &str,
) -> Result<String, ReminderServiceError> {
    recurrence.validate()?;

    if !recurrence.is_recurring() {
        return Err(ReminderServiceError::RecurrenceUnavailable);
    }

    let current = timestamp_millis(current_occurrence, "nextOccurrence")?;
    let anchor = timestamp_millis(anchor_scheduled_at, "scheduledAt")?;
    let fixed_interval = match recurrence {
        ReminderRecurrence::Hourly => Some(MILLISECONDS_PER_HOUR),
        ReminderRecurrence::Interval {
            unit: ReminderIntervalUnit::Minutes,
            value,
        } => Some(i64::from(*value) * MILLISECONDS_PER_MINUTE),
        ReminderRecurrence::Interval {
            unit: ReminderIntervalUnit::Hours,
            value,
        } => Some(i64::from(*value) * MILLISECONDS_PER_HOUR),
        _ => None,
    };
    let next = if let Some(interval) = fixed_interval {
        let elapsed = after_timestamp.saturating_sub(current);
        let interval_count = std::cmp::max(1, elapsed / interval + 1);
        current
            .checked_add(interval_count.saturating_mul(interval))
            .ok_or(ReminderServiceError::RecurrenceUnavailable)?
    } else if matches!(
        recurrence,
        ReminderRecurrence::Daily
            | ReminderRecurrence::Weekly
            | ReminderRecurrence::Interval {
                unit: ReminderIntervalUnit::Days,
                ..
            }
    ) {
        let interval_days = match recurrence {
            ReminderRecurrence::Daily => 1,
            ReminderRecurrence::Weekly => 7,
            ReminderRecurrence::Interval { value, .. } => u64::from(*value),
            _ => unreachable!(),
        };
        add_calendar_days(current, interval_days, after_timestamp)?
    } else {
        add_calendar_months(current, anchor, after_timestamp)?
    };

    format_timestamp(next).ok_or(ReminderServiceError::RecurrenceUnavailable)
}

fn add_calendar_days(
    current_timestamp: i64,
    interval_days: u64,
    after_timestamp: i64,
) -> Result<i64, ReminderServiceError> {
    let current = DateTime::<Utc>::from_timestamp_millis(current_timestamp)
        .ok_or(ReminderServiceError::RecurrenceUnavailable)?
        .with_timezone(&Local);
    let elapsed = after_timestamp.saturating_sub(current_timestamp).max(0);
    let estimated_intervals = std::cmp::max(
        1,
        u64::try_from(elapsed / (MILLISECONDS_PER_DAY * interval_days as i64)).unwrap_or(u64::MAX),
    );
    let mut days = estimated_intervals.saturating_mul(interval_days);

    loop {
        let date = current
            .date_naive()
            .checked_add_days(Days::new(days))
            .ok_or(ReminderServiceError::RecurrenceUnavailable)?;
        let naive = date
            .and_hms_nano_opt(
                current.hour(),
                current.minute(),
                current.second(),
                current.nanosecond(),
            )
            .ok_or(ReminderServiceError::RecurrenceUnavailable)?;
        let next = resolve_local_datetime(naive)?;
        let timestamp = next.timestamp_millis();

        if timestamp > current_timestamp && timestamp > after_timestamp {
            return Ok(timestamp);
        }

        days = days.saturating_add(interval_days);
    }
}

fn add_calendar_months(
    current_timestamp: i64,
    anchor_timestamp: i64,
    after_timestamp: i64,
) -> Result<i64, ReminderServiceError> {
    let current = DateTime::<Utc>::from_timestamp_millis(current_timestamp)
        .ok_or(ReminderServiceError::RecurrenceUnavailable)?
        .with_timezone(&Local);
    let anchor = DateTime::<Utc>::from_timestamp_millis(anchor_timestamp)
        .ok_or(ReminderServiceError::RecurrenceUnavailable)?
        .with_timezone(&Local);
    let after = DateTime::<Utc>::from_timestamp_millis(after_timestamp)
        .ok_or(ReminderServiceError::RecurrenceUnavailable)?
        .with_timezone(&Local);
    let estimated = std::cmp::max(
        1,
        (after.year() - current.year()) * 12 + after.month() as i32 - current.month() as i32,
    );
    let mut offset = u32::try_from(estimated).unwrap_or(u32::MAX);

    loop {
        let first = current
            .date_naive()
            .with_day(1)
            .and_then(|date| date.checked_add_months(Months::new(offset)))
            .ok_or(ReminderServiceError::RecurrenceUnavailable)?;
        let next_month = first
            .checked_add_months(Months::new(1))
            .ok_or(ReminderServiceError::RecurrenceUnavailable)?;
        let last_day = next_month
            .pred_opt()
            .ok_or(ReminderServiceError::RecurrenceUnavailable)?
            .day();
        let date = first
            .with_day(anchor.day().min(last_day))
            .ok_or(ReminderServiceError::RecurrenceUnavailable)?;
        let naive = date
            .and_hms_nano_opt(
                current.hour(),
                current.minute(),
                current.second(),
                current.nanosecond(),
            )
            .ok_or(ReminderServiceError::RecurrenceUnavailable)?;
        let next = resolve_local_datetime(naive)?;
        let timestamp = next.timestamp_millis();

        if timestamp > current_timestamp && timestamp > after_timestamp {
            return Ok(timestamp);
        }

        offset = offset
            .checked_add(1)
            .ok_or(ReminderServiceError::RecurrenceUnavailable)?;
    }
}

fn resolve_local_datetime(
    mut value: NaiveDateTime,
) -> Result<DateTime<Local>, ReminderServiceError> {
    for _ in 0..=180 {
        match Local.from_local_datetime(&value) {
            LocalResult::Single(value) => return Ok(value),
            LocalResult::Ambiguous(first, second) => return Ok(first.min(second)),
            LocalResult::None => {
                value = value
                    .checked_add_signed(chrono::Duration::minutes(1))
                    .ok_or(ReminderServiceError::RecurrenceUnavailable)?;
            }
        }
    }

    Err(ReminderServiceError::RecurrenceUnavailable)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicI64, Ordering as AtomicOrdering};

    use super::*;

    #[derive(Debug, Default)]
    struct MemoryRepository {
        reminders: Mutex<Vec<Reminder>>,
        fail_saves: Mutex<bool>,
    }

    impl ReminderRepository for MemoryRepository {
        fn list(&self) -> Result<Vec<Reminder>, ReminderRepositoryError> {
            self.reminders
                .lock()
                .map(|reminders| reminders.clone())
                .map_err(|_| ReminderRepositoryError::new("repository unavailable"))
        }

        fn save(&self, reminders: &[Reminder]) -> Result<(), ReminderRepositoryError> {
            if *self
                .fail_saves
                .lock()
                .map_err(|_| ReminderRepositoryError::new("repository unavailable"))?
            {
                return Err(ReminderRepositoryError::new("save failed"));
            }

            *self
                .reminders
                .lock()
                .map_err(|_| ReminderRepositoryError::new("repository unavailable"))? =
                reminders.to_vec();
            Ok(())
        }
    }

    #[derive(Debug, Default)]
    struct RecordingSink {
        notifications: Mutex<Vec<ReminderFiredNotification>>,
    }

    impl ReminderEventSink for RecordingSink {
        fn emit(&self, notification: ReminderFiredNotification) {
            self.notifications.lock().unwrap().push(notification);
        }
    }

    fn test_service(
        now: Arc<AtomicI64>,
        repository: Arc<MemoryRepository>,
        ids: Arc<Mutex<Vec<String>>>,
    ) -> Arc<ReminderService> {
        let clock = {
            let now = Arc::clone(&now);
            Arc::new(move || now.load(AtomicOrdering::SeqCst)) as Clock
        };
        let create_id = Arc::new(move || ids.lock().unwrap().remove(0)) as IdGenerator;
        Arc::new(ReminderService::with_dependencies(
            repository,
            clock,
            create_id,
            Arc::new(SchedulerWake::default()),
        ))
    }

    fn input(title: &str, scheduled_at: &str) -> CreateReminderInput {
        CreateReminderInput {
            title: title.to_owned(),
            message: None,
            scheduled_at: scheduled_at.to_owned(),
            recurrence: None,
        }
    }

    fn fired_notification(id: &str) -> ReminderFiredNotification {
        ReminderFiredNotification {
            reminder: Reminder {
                id: id.to_owned(),
                title: id.to_owned(),
                message: String::new(),
                scheduled_at: "2030-01-02T12:00:00.000Z".to_owned(),
                recurrence: ReminderRecurrence::None,
                last_triggered_at: None,
                next_occurrence: Some("2030-01-02T12:00:00.000Z".to_owned()),
                completed: false,
                created_at: "2030-01-01T12:00:00.000Z".to_owned(),
                updated_at: "2030-01-01T12:00:00.000Z".to_owned(),
            },
            fired_at: "2030-01-02T12:00:00.000Z".to_owned(),
            overdue: false,
        }
    }

    #[test]
    fn stored_reminders_preserve_electron_legacy_defaults() {
        let reminder = serde_json::from_value::<Reminder>(serde_json::json!({
            "id": "legacy",
            "title": "Legacy reminder",
            "message": "",
            "scheduledAt": "2030-01-02T12:00:00Z",
            "completed": false,
            "createdAt": "2030-01-01T12:00:00Z",
            "updatedAt": "2030-01-01T12:00:00Z"
        }))
        .expect("legacy reminder");

        assert_eq!(reminder.recurrence, ReminderRecurrence::None);
        assert_eq!(reminder.last_triggered_at, None);
        assert_eq!(
            reminder.next_occurrence.as_deref(),
            Some("2030-01-02T12:00:00.000Z")
        );

        let invalid_null_recurrence = serde_json::from_value::<Reminder>(serde_json::json!({
            "id": "invalid",
            "title": "Invalid reminder",
            "message": "",
            "scheduledAt": "2030-01-02T12:00:00Z",
            "recurrence": null,
            "completed": false,
            "createdAt": "2030-01-01T12:00:00Z",
            "updatedAt": "2030-01-01T12:00:00Z"
        }));
        assert!(invalid_null_recurrence.is_err());
    }

    #[test]
    fn pending_deliveries_survive_startup_reload_and_emit_in_order() {
        let delivered = Arc::new(Mutex::new(Vec::<String>::new()));
        let delivery_log = Arc::clone(&delivered);
        let queue = PendingReminderDeliveries::with_emitter(Arc::new(move |notification| {
            delivery_log.lock().unwrap().push(notification.reminder.id);
            Ok(())
        }));

        queue.emit(fired_notification("startup"));
        assert!(delivered.lock().unwrap().is_empty());

        queue.activate().unwrap();
        assert_eq!(*delivered.lock().unwrap(), ["startup"]);

        queue.deactivate();
        queue.emit(fired_notification("reload-first"));
        queue.emit(fired_notification("reload-second"));
        assert_eq!(*delivered.lock().unwrap(), ["startup"]);

        queue.activate().unwrap();
        assert_eq!(
            *delivered.lock().unwrap(),
            ["startup", "reload-first", "reload-second"]
        );
    }

    #[test]
    fn failed_delivery_stays_queued_until_the_next_activation() {
        let should_fail = Arc::new(Mutex::new(true));
        let delivered = Arc::new(Mutex::new(Vec::<String>::new()));
        let failure_state = Arc::clone(&should_fail);
        let delivery_log = Arc::clone(&delivered);
        let queue = PendingReminderDeliveries::with_emitter(Arc::new(move |notification| {
            if *failure_state.lock().unwrap() {
                return Err("renderer unavailable".to_owned());
            }

            delivery_log.lock().unwrap().push(notification.reminder.id);
            Ok(())
        }));
        queue.emit(fired_notification("retry"));

        queue.activate().unwrap();
        assert!(delivered.lock().unwrap().is_empty());

        *should_fail.lock().unwrap() = false;
        queue.activate().unwrap();
        assert_eq!(*delivered.lock().unwrap(), ["retry"]);
    }

    #[test]
    fn service_preserves_crud_validation_ordering_and_completion() {
        let now = Arc::new(AtomicI64::new(1_893_456_000_000));
        let repository = Arc::new(MemoryRepository::default());
        let service = test_service(
            Arc::clone(&now),
            Arc::clone(&repository),
            Arc::new(Mutex::new(vec!["later".to_owned(), "earlier".to_owned()])),
        );

        let later = service
            .create(input(
                "  Review release notes  ",
                "2030-01-03T05:30:00+05:30",
            ))
            .unwrap();
        let earlier = service
            .create(input("Stand up", "2030-01-02T00:00:00Z"))
            .unwrap();

        assert_eq!(later.title, "Review release notes");
        assert_eq!(later.scheduled_at, "2030-01-03T00:00:00.000Z");
        assert_eq!(service.list().unwrap()[0].id, earlier.id);
        now.store(1_893_459_600_000, AtomicOrdering::SeqCst);
        let updated = service
            .update(
                &earlier.id,
                UpdateReminderInput {
                    title: Some(" Daily stand-up ".to_owned()),
                    message: Some(" Team sync ".to_owned()),
                    scheduled_at: Some("2030-01-04T00:00:00Z".to_owned()),
                    recurrence: None,
                },
            )
            .unwrap();
        assert_eq!(updated.title, "Daily stand-up");
        assert_eq!(updated.message, "Team sync");
        assert_eq!(service.list().unwrap()[0].id, later.id);
        let completed = service.mark_completed(&earlier.id).unwrap();
        assert!(completed.completed);
        assert_eq!(completed.next_occurrence, None);
        assert_eq!(service.delete(&later.id).unwrap(), true);
        assert_eq!(service.delete("missing").unwrap(), false);
        assert_eq!(service.get("missing").unwrap(), None);
    }

    #[test]
    fn recurring_completion_preserves_id_and_advances_occurrence() {
        let now = Arc::new(AtomicI64::new(1_893_484_800_000));
        let repository = Arc::new(MemoryRepository::default());
        let service = test_service(
            Arc::clone(&now),
            repository,
            Arc::new(Mutex::new(vec!["recurring".to_owned()])),
        );
        let reminder = service
            .create(CreateReminderInput {
                recurrence: Some(ReminderRecurrence::Daily),
                ..input("Daily review", "2030-01-02T08:00:00Z")
            })
            .unwrap();
        now.store(1_893_571_200_000, AtomicOrdering::SeqCst);

        let advanced = service.mark_completed(&reminder.id).unwrap();

        assert_eq!(advanced.id, reminder.id);
        assert!(!advanced.completed);
        assert_eq!(
            advanced.next_occurrence.as_deref(),
            Some("2030-01-03T08:00:00.000Z")
        );
    }

    #[test]
    fn scheduler_fires_in_order_recovers_overdue_and_suppresses_retry_duplicates() {
        let now = Arc::new(AtomicI64::new(1_893_585_000_000));
        let repository = Arc::new(MemoryRepository::default());
        let service = test_service(
            Arc::clone(&now),
            Arc::clone(&repository),
            Arc::new(Mutex::new(vec!["overdue".to_owned(), "future".to_owned()])),
        );
        service
            .create(input("Overdue", "2030-01-02T11:59:55Z"))
            .unwrap();
        service
            .create(input("Future", "2030-01-02T12:00:20Z"))
            .unwrap();
        now.store(1_893_585_600_000, AtomicOrdering::SeqCst);
        *repository.fail_saves.lock().unwrap() = true;
        let sink = RecordingSink::default();
        let mut state = SchedulerState::default();

        let delay = state.synchronize(
            service.as_ref(),
            &sink,
            now.load(AtomicOrdering::SeqCst),
            CLOCK_VALIDATION_INTERVAL,
            MAXIMUM_OVERDUE_AGE,
        );
        assert_eq!(delay, Some(CLOCK_VALIDATION_INTERVAL));
        assert_eq!(sink.notifications.lock().unwrap().len(), 1);

        state.synchronize(
            service.as_ref(),
            &sink,
            now.load(AtomicOrdering::SeqCst),
            CLOCK_VALIDATION_INTERVAL,
            MAXIMUM_OVERDUE_AGE,
        );
        assert_eq!(sink.notifications.lock().unwrap().len(), 1);

        *repository.fail_saves.lock().unwrap() = false;
        state.synchronize(
            service.as_ref(),
            &sink,
            now.load(AtomicOrdering::SeqCst),
            CLOCK_VALIDATION_INTERVAL,
            MAXIMUM_OVERDUE_AGE,
        );
        assert_eq!(sink.notifications.lock().unwrap().len(), 1);
        assert!(service.get("overdue").unwrap().unwrap().completed);
    }

    #[test]
    fn scheduler_thread_starts_stops_and_is_singleton() {
        let now = Arc::new(AtomicI64::new(1_893_499_200_000));
        let repository = Arc::new(MemoryRepository::default());
        let service = test_service(
            now,
            repository,
            Arc::new(Mutex::new(vec!["unused".to_owned()])),
        );
        let sink = Arc::new(RecordingSink::default());
        let scheduler = ReminderScheduler::new(service, sink);

        scheduler.start().unwrap();
        scheduler.start().unwrap();
        assert!(scheduler.is_running());
        scheduler.stop().unwrap();
        assert!(!scheduler.is_running());
    }
}
