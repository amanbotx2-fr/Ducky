use serde::Serialize;
use tauri::{State, WebviewWindow};

use crate::{
    authorization,
    domain::{
        planner::{DailyPlannerBriefing, DailyPlannerService},
        reminders::ReminderRuntime,
        settings::SettingsState,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DailyPlannerCommandError {
    UnauthorizedWindow,
    PlannerUnavailable,
}

#[tauri::command]
pub(crate) fn get_daily_planner<R: tauri::Runtime>(
    window: WebviewWindow<R>,
    reminders: State<'_, ReminderRuntime>,
    settings: State<'_, SettingsState>,
) -> Result<DailyPlannerBriefing, DailyPlannerCommandError> {
    authorization::authorize_command(window.label(), authorization::GET_DAILY_PLANNER)
        .map_err(|_| DailyPlannerCommandError::UnauthorizedWindow)?;
    let user_name = settings
        .snapshot()
        .map_err(|_| DailyPlannerCommandError::PlannerUnavailable)?
        .user_name;

    DailyPlannerService::new(reminders.service.clone())
        .get_briefing(&user_name)
        .map_err(|_| DailyPlannerCommandError::PlannerUnavailable)
}
