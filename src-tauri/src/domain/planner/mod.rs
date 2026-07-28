use std::sync::Arc;

use chrono::{DateTime, Local};
use serde::Serialize;

use crate::domain::{
    reminders::{Reminder, ReminderService, ReminderServiceError},
    settings::normalize_user_name,
};

const MORNING_END_HOUR: u32 = 12;
const AFTERNOON_END_HOUR: u32 = 18;
const DEFAULT_USER_NAME: &str = "Friend";

type PlannerClock = Arc<dyn Fn() -> DateTime<Local> + Send + Sync>;

trait PlannerReminderSource: Send + Sync {
    fn list_reminders(&self) -> Result<Vec<Reminder>, ReminderServiceError>;
}

impl PlannerReminderSource for ReminderService {
    fn list_reminders(&self) -> Result<Vec<Reminder>, ReminderServiceError> {
        self.list()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DailyPlannerReminder {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) scheduled_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DailyPlannerBriefing {
    pub(crate) greeting: String,
    pub(crate) reminders: Vec<DailyPlannerReminder>,
}

#[derive(Debug)]
pub(crate) enum DailyPlannerError {
    Reminders(ReminderServiceError),
}

impl std::fmt::Display for DailyPlannerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reminders(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for DailyPlannerError {}

impl From<ReminderServiceError> for DailyPlannerError {
    fn from(error: ReminderServiceError) -> Self {
        Self::Reminders(error)
    }
}

pub(crate) struct DailyPlannerService {
    reminder_source: Arc<dyn PlannerReminderSource>,
    now: PlannerClock,
}

impl std::fmt::Debug for DailyPlannerService {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DailyPlannerService")
            .finish_non_exhaustive()
    }
}

impl DailyPlannerService {
    pub(crate) fn new(reminder_service: Arc<ReminderService>) -> Self {
        Self::with_clock(reminder_service, Arc::new(Local::now))
    }

    fn with_clock(reminder_source: Arc<dyn PlannerReminderSource>, now: PlannerClock) -> Self {
        Self {
            reminder_source,
            now,
        }
    }

    pub(crate) fn get_briefing(
        &self,
        user_name: &str,
    ) -> Result<DailyPlannerBriefing, DailyPlannerError> {
        let now = (self.now)();
        let mut reminders = self
            .reminder_source
            .list_reminders()?
            .into_iter()
            .filter_map(|reminder| to_planner_reminder(reminder, now))
            .collect::<Vec<_>>();

        reminders.sort_by(|left, right| {
            timestamp(&left.scheduled_at)
                .cmp(&timestamp(&right.scheduled_at))
                .then_with(|| left.id.cmp(&right.id))
        });

        Ok(DailyPlannerBriefing {
            greeting: greeting(user_name, now),
            reminders,
        })
    }
}

fn greeting(user_name: &str, now: DateTime<Local>) -> String {
    let normalized_name =
        normalize_user_name(user_name).unwrap_or_else(|| DEFAULT_USER_NAME.to_owned());
    let period = if now.hour() < MORNING_END_HOUR {
        "Morning"
    } else if now.hour() < AFTERNOON_END_HOUR {
        "Afternoon"
    } else {
        "Evening"
    };

    format!("Good {period}, {normalized_name}.")
}

fn to_planner_reminder(reminder: Reminder, now: DateTime<Local>) -> Option<DailyPlannerReminder> {
    if reminder.completed {
        return None;
    }

    let scheduled_at = reminder
        .next_occurrence
        .as_deref()
        .unwrap_or(&reminder.scheduled_at);
    let scheduled = DateTime::parse_from_rfc3339(scheduled_at)
        .ok()?
        .with_timezone(&Local);

    if scheduled < now || scheduled.date_naive() != now.date_naive() {
        return None;
    }

    Some(DailyPlannerReminder {
        id: reminder.id,
        title: reminder.title,
        scheduled_at: scheduled_at.to_owned(),
    })
}

fn timestamp(value: &str) -> i64 {
    DateTime::parse_from_rfc3339(value)
        .map(|date| date.timestamp_millis())
        .unwrap_or(i64::MAX)
}

use chrono::Timelike;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::{LocalResult, TimeZone, Utc};

    use super::*;
    use crate::domain::reminders::{ReminderIntervalUnit, ReminderRecurrence};

    #[derive(Debug)]
    struct TestReminderSource {
        reminders: Vec<Reminder>,
    }

    impl PlannerReminderSource for TestReminderSource {
        fn list_reminders(&self) -> Result<Vec<Reminder>, ReminderServiceError> {
            Ok(self.reminders.clone())
        }
    }

    fn local_date(hour: u32, minute: u32, day: u32) -> DateTime<Local> {
        match Local.with_ymd_and_hms(2030, 6, day, hour, minute, 0) {
            LocalResult::Single(value) => value,
            _ => panic!("test date must be unambiguous"),
        }
    }

    fn iso(date: DateTime<Local>) -> String {
        date.with_timezone(&Utc)
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
    }

    fn reminder(id: &str, scheduled: DateTime<Local>) -> Reminder {
        Reminder {
            id: id.to_owned(),
            title: format!("Reminder {id}"),
            message: String::new(),
            scheduled_at: iso(scheduled),
            recurrence: ReminderRecurrence::None,
            last_triggered_at: None,
            next_occurrence: Some(iso(scheduled)),
            completed: false,
            created_at: iso(local_date(8, 0, 1)),
            updated_at: iso(local_date(8, 0, 1)),
        }
    }

    fn planner(reminders: Vec<Reminder>, now: DateTime<Local>) -> DailyPlannerService {
        let source = Arc::new(TestReminderSource { reminders });

        DailyPlannerService::with_clock(source, Arc::new(move || now))
    }

    #[test]
    fn greeting_matches_electron_periods_and_name_normalization() {
        assert_eq!(
            planner(Vec::new(), local_date(11, 59, 15))
                .get_briefing("  Aman  ")
                .unwrap()
                .greeting,
            "Good Morning, Aman."
        );
        assert_eq!(
            planner(Vec::new(), local_date(12, 0, 15))
                .get_briefing("")
                .unwrap()
                .greeting,
            "Good Afternoon, Friend."
        );
        assert_eq!(
            planner(Vec::new(), local_date(18, 0, 15))
                .get_briefing(&"x".repeat(31))
                .unwrap()
                .greeting,
            "Good Evening, Friend."
        );
    }

    #[test]
    fn empty_schedule_returns_the_existing_empty_briefing() {
        let briefing = planner(Vec::new(), local_date(9, 0, 15))
            .get_briefing("Aman")
            .unwrap();

        assert!(briefing.reminders.is_empty());
    }

    #[test]
    fn upcoming_reminders_are_ordered_by_time_then_id() {
        let mut later = reminder("later", local_date(14, 0, 15));
        later.title = "Later".to_owned();
        let alpha = reminder("alpha", local_date(10, 30, 15));
        let beta = reminder("beta", local_date(10, 30, 15));
        let briefing = planner(vec![later, beta, alpha], local_date(9, 0, 15))
            .get_briefing("Aman")
            .unwrap();

        assert_eq!(
            briefing
                .reminders
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            ["alpha", "beta", "later"]
        );
    }

    #[test]
    fn recurring_reminder_uses_its_effective_next_occurrence() {
        let mut recurring = reminder("recurring", local_date(8, 0, 1));
        recurring.recurrence = ReminderRecurrence::Interval {
            unit: ReminderIntervalUnit::Days,
            value: 1,
        };
        recurring.last_triggered_at = Some(iso(local_date(16, 0, 14)));
        recurring.next_occurrence = Some(iso(local_date(16, 0, 15)));
        let briefing = planner(vec![recurring], local_date(9, 0, 15))
            .get_briefing("Aman")
            .unwrap();

        assert_eq!(
            briefing.reminders[0].scheduled_at,
            iso(local_date(16, 0, 15))
        );
    }

    #[test]
    fn completed_past_and_other_day_reminders_are_excluded() {
        let mut completed = reminder("completed", local_date(14, 0, 15));
        completed.completed = true;
        completed.next_occurrence = None;
        let past = reminder("past", local_date(8, 59, 15));
        let tomorrow = reminder("tomorrow", local_date(9, 0, 16));
        let upcoming = reminder("upcoming", local_date(11, 0, 15));
        let briefing = planner(
            vec![completed, past, tomorrow, upcoming],
            local_date(9, 0, 15),
        )
        .get_briefing("Aman")
        .unwrap();

        assert_eq!(briefing.reminders.len(), 1);
        assert_eq!(briefing.reminders[0].id, "upcoming");
    }

    #[test]
    fn malformed_effective_schedule_is_ignored_defensively() {
        let mut invalid = reminder("invalid", local_date(10, 0, 15));
        invalid.next_occurrence = Some("not-a-date".to_owned());
        let briefing = planner(vec![invalid], local_date(9, 0, 15))
            .get_briefing("Aman")
            .unwrap();

        assert!(briefing.reminders.is_empty());
    }
}
