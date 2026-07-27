use std::sync::Arc;

use chrono::{Local, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{AiFinishReason, AiResponse};
use crate::domain::{
    reminders::{CreateReminderInput, ReminderService},
    settings::{RuntimeSettings, SettingsState},
};

const MAXIMUM_PROMPT_CHARACTERS: usize = 4_096;
const MAXIMUM_CONTEXT_MESSAGES: usize = 16;
const MAXIMUM_CONTEXT_CHARACTERS: usize = 24_000;
const MAXIMUM_CONTEXT_MESSAGE_CHARACTERS: usize = 12_000;
const REMINDER_CREATED_MESSAGE: &str = "I've added that reminder.";
const STICKY_MESSAGE_UPDATED_MESSAGE: &str = "Sticky note updated.";

const ACTION_INSTRUCTIONS: &str = r#"You can perform exactly one of these local Ducky actions when the user explicitly requests it:

1. Create a reminder:
{"type":"createReminder","payload":{"title":"...","message":"...","scheduledAt":"ISO-8601 datetime with Z or an explicit offset","recurrence":{"type":"none"}}}

The optional recurrence value must be one of:
{"type":"none"}
{"type":"hourly"}
{"type":"daily"}
{"type":"weekly"}
{"type":"monthly"}
{"type":"interval","unit":"minutes|hours|days","value":positive integer}

2. Set the single sticky message:
{"type":"setStickyMessage","payload":{"message":"..."}}

When one supported action fully satisfies the request, return only its JSON object without Markdown or commentary. Never claim the action succeeded; the application confirms execution. If required details are missing or ambiguous, ask a concise clarification question instead. For all other requests, answer normally. Never invent or return other action types."#;

pub(crate) type RuntimeSettingsEmitter =
    Arc<dyn Fn(RuntimeSettings) -> Result<(), String> + Send + Sync>;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AiConversationRole {
    User,
    Assistant,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AiConversationContextMessage {
    role: AiConversationRole,
    content: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AiConversationRequest {
    prompt: String,
    history: Vec<AiConversationContextMessage>,
}

impl AiConversationRequest {
    pub(crate) fn validate(self) -> Result<Self, AssistantActionError> {
        let prompt = normalize_text(self.prompt, MAXIMUM_PROMPT_CHARACTERS)?;
        if self.history.len() > MAXIMUM_CONTEXT_MESSAGES {
            return Err(AssistantActionError::InvalidRequest);
        }

        let mut total_characters = 0;
        let mut history = Vec::with_capacity(self.history.len());
        for message in self.history {
            let content = normalize_text(message.content, MAXIMUM_CONTEXT_MESSAGE_CHARACTERS)?;
            total_characters += content.chars().count();
            if total_characters > MAXIMUM_CONTEXT_CHARACTERS {
                return Err(AssistantActionError::InvalidRequest);
            }
            history.push(AiConversationContextMessage {
                role: message.role,
                content,
            });
        }

        Ok(Self { prompt, history })
    }

    pub(crate) fn to_provider_prompt(&self) -> Result<String, AssistantActionError> {
        let time_zone = iana_time_zone::get_timezone().unwrap_or_else(|_| "UTC".to_owned());
        let local_now = Local::now();
        let utc_now = Utc::now();
        let local_date_time = local_now
            .format("%A, %B %-d, %Y at %H:%M:%S %:z")
            .to_string();
        self.to_provider_prompt_with_clock(
            &local_date_time,
            &time_zone,
            &utc_now.to_rfc3339_opts(SecondsFormat::Millis, true),
        )
    }

    fn to_provider_prompt_with_clock(
        &self,
        local_date_time: &str,
        time_zone: &str,
        utc_date_time: &str,
    ) -> Result<String, AssistantActionError> {
        let conversation_block = if self.history.is_empty() {
            String::new()
        } else {
            format!(
                "\n\nPrior conversation, oldest to newest:\n\
                 <conversation_history_json>\n{}\n\
                 </conversation_history_json>",
                serde_json::to_string(&self.history)
                    .map_err(|_| AssistantActionError::InvalidRequest)?
            )
        };

        Ok(format!(
            "{ACTION_INSTRUCTIONS}\n\n\
             Current local date and time: {local_date_time}\n\
             IANA timezone: {time_zone}\n\
             Current UTC time: {utc_date_time}{conversation_block}\n\n\
             <user_request>\n{}\n</user_request>",
            self.prompt
        ))
    }
}

#[derive(Clone)]
pub(crate) struct AssistantActionProcessor {
    reminders: Arc<ReminderService>,
    settings: SettingsState,
    emit_settings: RuntimeSettingsEmitter,
}

impl std::fmt::Debug for AssistantActionProcessor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AssistantActionProcessor")
            .finish_non_exhaustive()
    }
}

impl AssistantActionProcessor {
    pub(crate) fn new(
        reminders: Arc<ReminderService>,
        settings: SettingsState,
        emit_settings: RuntimeSettingsEmitter,
    ) -> Self {
        Self {
            reminders,
            settings,
            emit_settings,
        }
    }

    pub(crate) fn process(
        &self,
        mut response: AiResponse,
    ) -> Result<AiResponse, AssistantActionError> {
        match interpret_response(&response.content)? {
            AssistantResponse::Message(content) => {
                response.content = content;
                Ok(response)
            }
            AssistantResponse::Action(AssistantAction::CreateReminder(input)) => {
                self.reminders
                    .create(input)
                    .map_err(|_| AssistantActionError::ExecutionFailed)?;
                response.content = REMINDER_CREATED_MESSAGE.to_owned();
                response.finish_reason = AiFinishReason::Stop;
                Ok(response)
            }
            AssistantResponse::Action(AssistantAction::SetStickyMessage(message)) => {
                let update = self
                    .settings
                    .update_sticky_message(Some(message))
                    .map_err(|_| AssistantActionError::ExecutionFailed)?;
                if update.changed {
                    (self.emit_settings)(update.settings.runtime_projection())
                        .map_err(|_| AssistantActionError::ExecutionFailed)?;
                }
                response.content = STICKY_MESSAGE_UPDATED_MESSAGE.to_owned();
                response.finish_reason = AiFinishReason::Stop;
                Ok(response)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AssistantActionError {
    InvalidRequest,
    InvalidAction,
    InvalidPayload,
    UnknownAction,
    ExecutionFailed,
}

enum AssistantAction {
    CreateReminder(CreateReminderInput),
    SetStickyMessage(String),
}

enum AssistantResponse {
    Message(String),
    Action(AssistantAction),
}

fn interpret_response(content: &str) -> Result<AssistantResponse, AssistantActionError> {
    let normalized = content.trim();
    let candidate = unwrap_json_code_fence(normalized);
    if !candidate.starts_with('{') {
        return Ok(AssistantResponse::Message(normalized.to_owned()));
    }

    let value: Value = match serde_json::from_str(candidate) {
        Ok(value) => value,
        Err(_) if looks_like_action(candidate) => {
            return Err(AssistantActionError::InvalidAction);
        }
        Err(_) => return Ok(AssistantResponse::Message(normalized.to_owned())),
    };
    let Some(object) = value.as_object() else {
        return Ok(AssistantResponse::Message(normalized.to_owned()));
    };
    if !object.contains_key("type") {
        return Ok(AssistantResponse::Message(normalized.to_owned()));
    }
    if object.len() != 2 || !object.contains_key("payload") {
        return Err(AssistantActionError::InvalidAction);
    }
    let action_type = object
        .get("type")
        .and_then(Value::as_str)
        .ok_or(AssistantActionError::InvalidAction)?;
    let payload = object
        .get("payload")
        .cloned()
        .ok_or(AssistantActionError::InvalidAction)?;

    match action_type {
        "createReminder" => serde_json::from_value(payload)
            .map(AssistantAction::CreateReminder)
            .map_err(|_| AssistantActionError::InvalidPayload),
        "setStickyMessage" => parse_sticky_message(payload),
        _ => Err(AssistantActionError::UnknownAction),
    }
    .map(AssistantResponse::Action)
}

fn parse_sticky_message(value: Value) -> Result<AssistantAction, AssistantActionError> {
    let object = value
        .as_object()
        .ok_or(AssistantActionError::InvalidPayload)?;
    if object.len() != 1 {
        return Err(AssistantActionError::InvalidPayload);
    }
    object
        .get("message")
        .and_then(Value::as_str)
        .map(|message| AssistantAction::SetStickyMessage(message.to_owned()))
        .ok_or(AssistantActionError::InvalidPayload)
}

fn unwrap_json_code_fence(content: &str) -> &str {
    let Some(first_newline) = content.find('\n') else {
        return content;
    };
    let opening = content[..first_newline].trim().to_ascii_lowercase();
    if opening != "```" && opening != "```json" {
        return content;
    }
    let remainder = &content[first_newline + 1..];
    let Some(body) = remainder.strip_suffix("```") else {
        return content;
    };
    body.trim()
}

fn looks_like_action(content: &str) -> bool {
    content
        .strip_prefix('{')
        .map(str::trim_start)
        .is_some_and(|content| {
            content
                .strip_prefix("\"type\"")
                .map(str::trim_start)
                .is_some_and(|content| content.starts_with(':'))
        })
}

fn normalize_text(value: String, maximum: usize) -> Result<String, AssistantActionError> {
    if value.chars().count() > maximum {
        return Err(AssistantActionError::InvalidRequest);
    }
    let normalized = value.trim().to_owned();
    if normalized.is_empty() {
        Err(AssistantActionError::InvalidRequest)
    } else {
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::super::provider::AiUsage;
    use super::*;
    use crate::{
        domain::{ai::AiProviderId, reminders::ReminderRuntime, settings::SettingsDocument},
        infrastructure::persistence::SettingsStore,
    };

    fn request() -> AiConversationRequest {
        AiConversationRequest {
            prompt: " Remind me tomorrow ".to_owned(),
            history: vec![AiConversationContextMessage {
                role: AiConversationRole::Assistant,
                content: " What time? ".to_owned(),
            }],
        }
        .validate()
        .unwrap()
    }

    fn response(content: &str) -> AiResponse {
        AiResponse {
            provider_id: AiProviderId::Openai,
            content: content.to_owned(),
            finish_reason: AiFinishReason::Length,
            usage: Some(AiUsage {
                input_tokens: 4,
                output_tokens: 2,
            }),
        }
    }

    fn processor() -> (AssistantActionProcessor, SettingsState) {
        let directory = tempdir().unwrap().keep();
        let settings = SettingsState::new(
            SettingsStore::new(directory.join("settings.json")),
            SettingsDocument::default(),
        );
        let reminders =
            ReminderRuntime::with_delivery(Arc::new(settings.clone()), Arc::new(|_| Ok(())));
        (
            AssistantActionProcessor::new(
                reminders.service,
                settings.clone(),
                Arc::new(|_| Ok(())),
            ),
            settings,
        )
    }

    #[test]
    fn prompt_matches_the_existing_action_contract_and_conversation_shape() {
        let prompt = request()
            .to_provider_prompt_with_clock(
                "Monday, July 28, 2026 at 12:34:56 +05:30",
                "Asia/Kolkata",
                "2026-07-28T07:04:56.000Z",
            )
            .unwrap();

        assert!(prompt.contains(r#"{"type":"createReminder""#));
        assert!(prompt.contains(r#"{"type":"setStickyMessage""#));
        assert!(prompt.contains("IANA timezone: Asia/Kolkata"));
        assert!(prompt.contains(r#""role":"assistant","content":"What time?""#));
        assert!(prompt.ends_with("<user_request>\nRemind me tomorrow\n</user_request>"));
    }

    #[test]
    fn ordinary_and_unrelated_json_responses_remain_messages() {
        let (processor, _) = processor();
        let ordinary = processor.process(response("  Hello  ")).unwrap();
        assert_eq!(ordinary.content, "Hello");
        assert_eq!(ordinary.finish_reason, AiFinishReason::Length);
        let json = processor
            .process(response(r#"{"answer":"ordinary JSON"}"#))
            .unwrap();
        assert_eq!(json.content, r#"{"answer":"ordinary JSON"}"#);
    }

    #[test]
    fn exact_allowlist_executes_reminder_and_sticky_actions() {
        let (processor, settings) = processor();
        let reminder = processor
            .process(response(
                r#"{"type":"createReminder","payload":{"title":"Test","scheduledAt":"2099-01-01T00:00:00Z","recurrence":{"type":"none"}}}"#,
            ))
            .unwrap();
        assert_eq!(reminder.content, REMINDER_CREATED_MESSAGE);
        assert_eq!(settings.snapshot().unwrap().reminders.len(), 1);

        let sticky = processor
            .process(response(
                r#"```json
{"type":"setStickyMessage","payload":{"message":"Ship Phase 9"}}
```"#,
            ))
            .unwrap();
        assert_eq!(sticky.content, STICKY_MESSAGE_UPDATED_MESSAGE);
        assert_eq!(
            settings.snapshot().unwrap().sticky_message.as_deref(),
            Some("Ship Phase 9")
        );
    }

    #[test]
    fn malformed_unknown_and_extra_capability_actions_are_rejected() {
        let (processor, settings) = processor();
        for content in [
            r#"{"type":"createReminder","payload":{"title":"missing date"}}"#,
            r#"{"type":"deleteReminder","payload":{"id":"all"}}"#,
            r#"{"type":"setStickyMessage","payload":{"message":"x","admin":true}}"#,
            r#"{"type":"setStickyMessage","payload":{"message":"x"}"#,
        ] {
            assert!(processor.process(response(content)).is_err());
        }
        assert!(settings.snapshot().unwrap().reminders.is_empty());
        assert!(settings.snapshot().unwrap().sticky_message.is_none());
    }
}
