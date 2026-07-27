use std::{
    net::IpAddr,
    sync::{Mutex, PoisonError, RwLock},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use url::Url;

use crate::infrastructure::persistence::SettingsStore;

const DEFAULT_USER_NAME: &str = "Friend";
const DEFAULT_OLLAMA_ENDPOINT: &str = "http://localhost:11434";
const MAXIMUM_USER_NAME_LENGTH: usize = 30;
const MAXIMUM_STICKY_MESSAGE_LENGTH: usize = 120;
const MAXIMUM_MODEL_LENGTH: usize = 256;
const MAXIMUM_ENDPOINT_LENGTH: usize = 2_048;
const MAXIMUM_FAVORITE_MODELS: usize = 512;
const MAXIMUM_RECENT_MODELS: usize = 25;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SettingsDocument {
    #[serde(default = "default_user_name")]
    pub(crate) user_name: String,
    #[serde(default)]
    pub(crate) sticky_message: Option<String>,
    #[serde(default)]
    pub(crate) reminders: Vec<Value>,
    pub(crate) general: GeneralSettings,
    pub(crate) water: WaterSettings,
    #[serde(default)]
    pub(crate) notification_sounds: NotificationSoundSettings,
    #[serde(default)]
    pub(crate) updates: UpdateSettings,
    pub(crate) ai: StoredAiSettings,
    #[serde(default)]
    pub(crate) ai_model_explorer: AiModelExplorerSettings,
    #[serde(default)]
    pub(crate) credential: Option<Value>,
}

impl Default for SettingsDocument {
    fn default() -> Self {
        Self {
            user_name: default_user_name(),
            sticky_message: None,
            reminders: Vec::new(),
            general: GeneralSettings::default(),
            water: WaterSettings::default(),
            notification_sounds: NotificationSoundSettings::default(),
            updates: UpdateSettings::default(),
            ai: StoredAiSettings::default(),
            ai_model_explorer: AiModelExplorerSettings::default(),
            credential: None,
        }
    }
}

impl SettingsDocument {
    pub(crate) fn parse(value: Value) -> Result<Self, SettingsValidationError> {
        let mut settings: Self = serde_json::from_value(value)
            .map_err(|error| SettingsValidationError::new(error.to_string()))?;
        settings.validate_and_canonicalize()?;
        Ok(settings)
    }

    pub(crate) fn validate(&self) -> Result<(), SettingsValidationError> {
        let mut copy = self.clone();
        copy.validate_and_canonicalize()
    }

    fn validate_and_canonicalize(&mut self) -> Result<(), SettingsValidationError> {
        self.user_name =
            normalize_required_text(&self.user_name, MAXIMUM_USER_NAME_LENGTH, "userName")?;

        if let Some(sticky_message) = self.sticky_message.as_mut() {
            *sticky_message = normalize_required_text(
                sticky_message,
                MAXIMUM_STICKY_MESSAGE_LENGTH,
                "stickyMessage",
            )?;
        }

        if !self.reminders.iter().all(Value::is_object) {
            return Err(SettingsValidationError::new(
                "reminders must contain objects",
            ));
        }

        self.water.validate()?;
        self.notification_sounds.validate()?;
        self.ai.validate_and_canonicalize()?;
        self.ai_model_explorer.validate_and_canonicalize()?;

        if self
            .credential
            .as_ref()
            .is_some_and(|credential| !credential.is_object())
        {
            return Err(SettingsValidationError::new(
                "credential must be an object or null",
            ));
        }

        Ok(())
    }

    pub(crate) fn runtime_projection(&self) -> RuntimeSettings {
        RuntimeSettings {
            user_name: self.user_name.clone(),
            sticky_message: self.sticky_message.clone(),
            general: self.general.clone(),
            water: self.water.clone(),
            notification_sounds: self.notification_sounds.clone(),
        }
    }

    pub(crate) fn preferences_projection(&self) -> PreferencesSettings {
        PreferencesSettings {
            user_name: self.user_name.clone(),
            general: self.general.clone(),
            water: self.water.clone(),
            notification_sounds: self.notification_sounds.clone(),
            updates: self.updates.clone(),
            ai: PreferencesAiSettings {
                enabled: self.ai.enabled,
                provider: self.ai.provider.clone(),
                model: self.ai.model.clone(),
                api_key_configured: self.credential.is_some()
                    || self
                        .ai
                        .api_key
                        .as_ref()
                        .is_some_and(|api_key| !api_key.trim().is_empty()),
                endpoint: self.ai.endpoint.clone(),
                base_url: self.ai.base_url.clone(),
            },
            ai_model_explorer: self.ai_model_explorer.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct SettingsState {
    pub(crate) store: SettingsStore,
    pub(crate) settings: RwLock<SettingsDocument>,
    mutation_lock: Mutex<()>,
}

impl SettingsState {
    pub(crate) fn new(store: SettingsStore, settings: SettingsDocument) -> Self {
        Self {
            store,
            settings: RwLock::new(settings),
            mutation_lock: Mutex::new(()),
        }
    }

    pub(crate) fn snapshot(&self) -> Result<SettingsDocument, SettingsStateError> {
        self.settings
            .read()
            .map(|settings| settings.clone())
            .map_err(SettingsStateError::from)
    }

    pub(crate) fn update_user_name(
        &self,
        value: String,
    ) -> Result<SettingsUpdate, SettingsMutationError> {
        let user_name = normalize_required_text(&value, MAXIMUM_USER_NAME_LENGTH, "userName")?;
        self.persist_update(move |settings| {
            let mut next = settings.clone();
            next.user_name = user_name;
            next
        })
    }

    pub(crate) fn update_sticky_message(
        &self,
        value: Option<String>,
    ) -> Result<SettingsUpdate, SettingsMutationError> {
        let sticky_message = value
            .map(|message| {
                normalize_required_text(&message, MAXIMUM_STICKY_MESSAGE_LENGTH, "stickyMessage")
            })
            .transpose()?;

        self.persist_update(move |settings| {
            let mut next = settings.clone();
            next.sticky_message = sticky_message;
            next
        })
    }

    pub(crate) fn update_preferences(
        &self,
        patch: PreferencesSettingsPatch,
    ) -> Result<SettingsUpdate, SettingsMutationError> {
        self.persist_update(move |settings| {
            let mut next = settings.clone();

            if let Some(general) = patch.general {
                if let Some(always_on_top) = general.always_on_top {
                    next.general.always_on_top = always_on_top;
                }
                if let Some(launch_at_startup) = general.launch_at_startup {
                    next.general.launch_at_startup = launch_at_startup;
                }
                if let Some(eye_tracking) = general.eye_tracking {
                    next.general.eye_tracking = eye_tracking;
                }
            }

            if let Some(notification_sounds) = patch.notification_sounds {
                if let Some(enabled) = notification_sounds.enabled {
                    next.notification_sounds.enabled = enabled;
                }
                if let Some(sound) = notification_sounds.sound {
                    next.notification_sounds.sound = sound;
                }
                if let Some(volume) = notification_sounds.volume {
                    next.notification_sounds.volume = volume;
                }
            }

            next
        })
    }

    fn persist_update(
        &self,
        update: impl FnOnce(&SettingsDocument) -> SettingsDocument,
    ) -> Result<SettingsUpdate, SettingsMutationError> {
        let _mutation_guard = self
            .mutation_lock
            .lock()
            .map_err(SettingsStateError::from)?;
        let current = self.snapshot()?;
        let next = update(&current);
        next.validate()?;

        if next == current {
            return Ok(SettingsUpdate {
                settings: current,
                changed: false,
            });
        }

        // The read/write lock is deliberately not held across filesystem I/O.
        // The mutation mutex serializes writers while snapshots remain cheap.
        self.store.save(&next)?;
        *self.settings.write().map_err(SettingsStateError::from)? = next.clone();

        Ok(SettingsUpdate {
            settings: next,
            changed: true,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RuntimeSettings {
    pub(crate) user_name: String,
    pub(crate) sticky_message: Option<String>,
    pub(crate) general: GeneralSettings,
    pub(crate) water: WaterSettings,
    pub(crate) notification_sounds: NotificationSoundSettings,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreferencesSettings {
    pub(crate) user_name: String,
    pub(crate) general: GeneralSettings,
    pub(crate) water: WaterSettings,
    pub(crate) notification_sounds: NotificationSoundSettings,
    pub(crate) updates: UpdateSettings,
    pub(crate) ai: PreferencesAiSettings,
    pub(crate) ai_model_explorer: AiModelExplorerSettings,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreferencesAiSettings {
    pub(crate) enabled: bool,
    pub(crate) provider: String,
    pub(crate) model: String,
    pub(crate) api_key_configured: bool,
    pub(crate) endpoint: String,
    pub(crate) base_url: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PreferencesSettingsPatch {
    #[serde(default)]
    pub(crate) general: Option<GeneralSettingsPatch>,
    #[serde(default)]
    pub(crate) notification_sounds: Option<NotificationSoundSettingsPatch>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct GeneralSettingsPatch {
    #[serde(default)]
    pub(crate) always_on_top: Option<bool>,
    #[serde(default)]
    pub(crate) launch_at_startup: Option<bool>,
    #[serde(default)]
    pub(crate) eye_tracking: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct NotificationSoundSettingsPatch {
    #[serde(default)]
    pub(crate) enabled: Option<bool>,
    #[serde(default)]
    pub(crate) sound: Option<NotificationSoundId>,
    #[serde(default)]
    pub(crate) volume: Option<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SettingsUpdate {
    pub(crate) settings: SettingsDocument,
    pub(crate) changed: bool,
}

#[derive(Debug)]
pub(crate) enum SettingsMutationError {
    State(SettingsStateError),
    Validation(SettingsValidationError),
    Store(crate::infrastructure::persistence::SettingsStoreError),
}

impl std::fmt::Display for SettingsMutationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::State(error) => error.fmt(formatter),
            Self::Validation(error) => error.fmt(formatter),
            Self::Store(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for SettingsMutationError {}

impl From<SettingsStateError> for SettingsMutationError {
    fn from(error: SettingsStateError) -> Self {
        Self::State(error)
    }
}

impl From<SettingsValidationError> for SettingsMutationError {
    fn from(error: SettingsValidationError) -> Self {
        Self::Validation(error)
    }
}

impl From<crate::infrastructure::persistence::SettingsStoreError> for SettingsMutationError {
    fn from(error: crate::infrastructure::persistence::SettingsStoreError) -> Self {
        Self::Store(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SettingsStateError;

impl<T> From<PoisonError<T>> for SettingsStateError {
    fn from(_error: PoisonError<T>) -> Self {
        Self
    }
}

impl std::fmt::Display for SettingsStateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("settings state is unavailable")
    }
}

impl std::error::Error for SettingsStateError {}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct GeneralSettings {
    pub(crate) always_on_top: bool,
    pub(crate) launch_at_startup: bool,
    pub(crate) eye_tracking: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            always_on_top: true,
            launch_at_startup: false,
            eye_tracking: true,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct WaterSettings {
    pub(crate) enabled: bool,
    pub(crate) interval: u16,
}

impl Default for WaterSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 30,
        }
    }
}

impl WaterSettings {
    fn validate(&self) -> Result<(), SettingsValidationError> {
        if [15, 30, 45, 60, 90, 120].contains(&self.interval) {
            Ok(())
        } else {
            Err(SettingsValidationError::new(
                "water.interval is not supported",
            ))
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum NotificationSoundId {
    #[default]
    SoftBell,
    DigitalBell,
    ZenChime,
    Pop,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct NotificationSoundSettings {
    pub(crate) enabled: bool,
    pub(crate) sound: NotificationSoundId,
    pub(crate) volume: u8,
}

impl Default for NotificationSoundSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            sound: NotificationSoundId::SoftBell,
            volume: 70,
        }
    }
}

impl NotificationSoundSettings {
    fn validate(&self) -> Result<(), SettingsValidationError> {
        if self.volume <= 100 {
            Ok(())
        } else {
            Err(SettingsValidationError::new(
                "notificationSounds.volume must be between 0 and 100",
            ))
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UpdateSettings {
    pub(crate) automatic: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StoredAiSettings {
    pub(crate) enabled: bool,
    pub(crate) provider: String,
    #[serde(default)]
    pub(crate) model: String,
    #[serde(default = "default_ollama_endpoint")]
    pub(crate) endpoint: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(crate) base_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) api_key: Option<String>,
}

impl Default for StoredAiSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: String::new(),
            model: String::new(),
            endpoint: default_ollama_endpoint(),
            base_url: String::new(),
            api_key: None,
        }
    }
}

impl StoredAiSettings {
    fn validate_and_canonicalize(&mut self) -> Result<(), SettingsValidationError> {
        if !["", "openai", "gemini", "grok", "ollama", "custom"].contains(&self.provider.as_str()) {
            return Err(SettingsValidationError::new("ai.provider is not supported"));
        }

        if self.model.len() > MAXIMUM_MODEL_LENGTH {
            return Err(SettingsValidationError::new("ai.model is too long"));
        }

        validate_http_url(&self.endpoint, false, "ai.endpoint")?;

        if !self.base_url.is_empty() {
            self.base_url = normalize_custom_ai_base_url(&self.base_url)?;
        }

        if self
            .api_key
            .as_ref()
            .is_some_and(|api_key| api_key.len() > 4_096)
        {
            return Err(SettingsValidationError::new("ai.apiKey is too long"));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AiModelExplorerSettings {
    #[serde(default)]
    pub(crate) favorites: Vec<AiModelReference>,
    #[serde(default)]
    pub(crate) recent: Vec<AiModelReference>,
}

impl AiModelExplorerSettings {
    fn validate_and_canonicalize(&mut self) -> Result<(), SettingsValidationError> {
        validate_model_references(
            &mut self.favorites,
            MAXIMUM_FAVORITE_MODELS,
            "aiModelExplorer.favorites",
        )?;
        validate_model_references(
            &mut self.recent,
            MAXIMUM_RECENT_MODELS,
            "aiModelExplorer.recent",
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AiModelReference {
    pub(crate) provider: String,
    pub(crate) model_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SettingsValidationError {
    message: String,
}

impl SettingsValidationError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for SettingsValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SettingsValidationError {}

fn default_user_name() -> String {
    DEFAULT_USER_NAME.to_owned()
}

fn default_ollama_endpoint() -> String {
    DEFAULT_OLLAMA_ENDPOINT.to_owned()
}

fn normalize_required_text(
    value: &str,
    maximum_length: usize,
    field: &str,
) -> Result<String, SettingsValidationError> {
    let normalized = value.trim();

    if normalized.is_empty() || normalized.len() > maximum_length {
        return Err(SettingsValidationError::new(format!("{field} is invalid")));
    }

    Ok(normalized.to_owned())
}

fn validate_http_url(
    value: &str,
    allow_empty: bool,
    field: &str,
) -> Result<Url, SettingsValidationError> {
    if allow_empty && value.is_empty() {
        return Url::parse("https://invalid.local")
            .map_err(|error| SettingsValidationError::new(error.to_string()));
    }

    if value.len() > MAXIMUM_ENDPOINT_LENGTH {
        return Err(SettingsValidationError::new(format!("{field} is too long")));
    }

    let url = Url::parse(value)
        .map_err(|_| SettingsValidationError::new(format!("{field} is invalid")))?;

    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(SettingsValidationError::new(format!("{field} is invalid")));
    }

    Ok(url)
}

fn normalize_custom_ai_base_url(value: &str) -> Result<String, SettingsValidationError> {
    let url = validate_http_url(value.trim(), false, "ai.baseUrl")?;
    let host = url
        .host_str()
        .ok_or_else(|| SettingsValidationError::new("ai.baseUrl has no host"))?;

    if url.scheme() == "http" && !is_local_or_private_host(host) {
        return Err(SettingsValidationError::new(
            "ai.baseUrl requires HTTPS outside a local network",
        ));
    }

    let mut normalized = format!("{}://{}", url.scheme(), url.host_str().unwrap_or_default());

    if let Some(port) = url.port() {
        normalized.push(':');
        normalized.push_str(&port.to_string());
    }

    let path = url.path().trim_end_matches('/');
    if !path.is_empty() {
        normalized.push_str(path);
    }

    Ok(normalized)
}

fn is_local_or_private_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    match host.parse::<IpAddr>() {
        Ok(IpAddr::V4(address)) => address.is_loopback() || address.is_private(),
        Ok(IpAddr::V6(address)) => address.is_loopback() || address.is_unique_local(),
        Err(_) => false,
    }
}

fn validate_model_references(
    references: &mut Vec<AiModelReference>,
    maximum_length: usize,
    field: &str,
) -> Result<(), SettingsValidationError> {
    if references.len() > maximum_length {
        return Err(SettingsValidationError::new(format!(
            "{field} has too many entries"
        )));
    }

    let mut unique = Vec::with_capacity(references.len());
    for mut reference in references.drain(..) {
        if !["openai", "gemini", "grok", "ollama", "custom"].contains(&reference.provider.as_str())
        {
            return Err(SettingsValidationError::new(format!(
                "{field} contains an unsupported provider"
            )));
        }

        reference.model_id =
            normalize_required_text(&reference.model_id, MAXIMUM_MODEL_LENGTH, field)?;

        if !unique.iter().any(|existing: &AiModelReference| {
            existing.provider == reference.provider && existing.model_id == reference.model_id
        }) {
            unique.push(reference);
        }
    }

    *references = unique;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn defaults_match_the_electron_settings_contract() {
        let defaults = SettingsDocument::default();

        assert_eq!(defaults.user_name, "Friend");
        assert_eq!(defaults.sticky_message, None);
        assert!(defaults.reminders.is_empty());
        assert_eq!(
            defaults.general,
            GeneralSettings {
                always_on_top: true,
                launch_at_startup: false,
                eye_tracking: true,
            }
        );
        assert_eq!(defaults.water, WaterSettings::default());
        assert_eq!(
            defaults.notification_sounds,
            NotificationSoundSettings::default()
        );
        assert_eq!(defaults.updates, UpdateSettings { automatic: false });
        assert_eq!(defaults.ai, StoredAiSettings::default());
        assert_eq!(
            defaults.ai_model_explorer,
            AiModelExplorerSettings::default()
        );
        assert_eq!(defaults.credential, None);
    }

    #[test]
    fn missing_additive_fields_receive_canonical_defaults() {
        let settings = SettingsDocument::parse(json!({
            "general": {
                "alwaysOnTop": false,
                "launchAtStartup": true,
                "eyeTracking": false
            },
            "water": {
                "enabled": false,
                "interval": 45
            },
            "ai": {
                "enabled": false,
                "provider": ""
            }
        }))
        .expect("legacy settings should parse");

        assert_eq!(settings.user_name, "Friend");
        assert_eq!(settings.notification_sounds.volume, 70);
        assert_eq!(settings.ai.endpoint, DEFAULT_OLLAMA_ENDPOINT);
    }

    #[test]
    fn settings_validation_rejects_unknown_and_invalid_values() {
        let unknown = serde_json::from_value::<SettingsDocument>(json!({
            "general": {
                "alwaysOnTop": true,
                "launchAtStartup": false,
                "eyeTracking": true
            },
            "water": { "enabled": true, "interval": 30 },
            "ai": { "enabled": false, "provider": "" },
            "unexpected": true
        }));
        assert!(unknown.is_err());

        let invalid = SettingsDocument::parse(json!({
            "general": {
                "alwaysOnTop": true,
                "launchAtStartup": false,
                "eyeTracking": true
            },
            "water": { "enabled": true, "interval": 31 },
            "ai": { "enabled": false, "provider": "" }
        }));
        assert!(invalid.is_err());
    }

    #[test]
    fn shared_electron_fixture_matches_the_native_schema() {
        let fixture = include_str!("../../../../tests/fixtures/settings/electron-current.json");
        let value = serde_json::from_str(fixture).expect("fixture JSON");
        let settings = SettingsDocument::parse(value).expect("native settings fixture");

        assert_eq!(settings.user_name, "Aman");
        assert_eq!(
            settings.notification_sounds.sound,
            NotificationSoundId::ZenChime
        );
        assert_eq!(settings.notification_sounds.volume, 42);
        assert_eq!(settings.general.always_on_top, false);
    }

    #[test]
    fn deferred_data_is_preserved_without_becoming_renderer_state() {
        let credential = json!({
            "version": 1,
            "ciphertext": "dGVzdA=="
        });
        let reminder = json!({
            "id": "deferred-reminder"
        });
        let mut settings = SettingsDocument::default();
        settings.credential = Some(credential.clone());
        settings.reminders.push(reminder.clone());

        settings.validate().expect("opaque deferred data is valid");
        let serialized = serde_json::to_value(&settings).expect("settings serialize");

        assert_eq!(serialized["credential"], credential);
        assert_eq!(serialized["reminders"][0], reminder);
    }

    #[test]
    fn mutations_persist_before_replacing_the_shared_snapshot() {
        let directory = tempdir().expect("temporary directory");
        let store = SettingsStore::new(directory.path().join("settings.json"));
        store
            .save(&SettingsDocument::default())
            .expect("initial settings save");
        let state = SettingsState::new(store.clone(), SettingsDocument::default());

        let update = state
            .update_preferences(PreferencesSettingsPatch {
                general: Some(GeneralSettingsPatch {
                    always_on_top: Some(false),
                    ..GeneralSettingsPatch::default()
                }),
                notification_sounds: Some(NotificationSoundSettingsPatch {
                    volume: Some(25),
                    ..NotificationSoundSettingsPatch::default()
                }),
            })
            .expect("settings mutation");

        assert!(update.changed);
        assert!(!state.snapshot().unwrap().general.always_on_top);
        assert_eq!(store.load().unwrap().notification_sounds.volume, 25);
    }

    #[test]
    fn no_op_mutations_skip_persistence() {
        let directory = tempdir().expect("temporary directory");
        let blocked_parent = directory.path().join("not-a-directory");
        std::fs::write(&blocked_parent, "blocked").expect("blocking file");
        let store = SettingsStore::new(blocked_parent.join("settings.json"));
        let defaults = SettingsDocument::default();
        let state = SettingsState::new(store, defaults);

        let update = state
            .update_preferences(PreferencesSettingsPatch {
                general: Some(GeneralSettingsPatch {
                    always_on_top: Some(true),
                    ..GeneralSettingsPatch::default()
                }),
                notification_sounds: None,
            })
            .expect("no-op mutation");

        assert!(!update.changed);
        assert!(matches!(
            state.update_user_name("Changed".to_owned()),
            Err(SettingsMutationError::Store(_))
        ));
    }

    #[test]
    fn concurrent_mutations_are_serialized_without_lost_updates() {
        let directory = tempdir().expect("temporary directory");
        let store = SettingsStore::new(directory.path().join("settings.json"));
        store
            .save(&SettingsDocument::default())
            .expect("initial settings save");
        let state = Arc::new(SettingsState::new(store, SettingsDocument::default()));
        let name_state = Arc::clone(&state);
        let sound_state = Arc::clone(&state);

        let name_update =
            std::thread::spawn(move || name_state.update_user_name("Aman".to_owned()));
        let sound_update = std::thread::spawn(move || {
            sound_state.update_preferences(PreferencesSettingsPatch {
                general: None,
                notification_sounds: Some(NotificationSoundSettingsPatch {
                    sound: Some(NotificationSoundId::Pop),
                    ..NotificationSoundSettingsPatch::default()
                }),
            })
        });

        name_update.join().unwrap().unwrap();
        sound_update.join().unwrap().unwrap();
        let snapshot = state.snapshot().unwrap();
        assert_eq!(snapshot.user_name, "Aman");
        assert_eq!(snapshot.notification_sounds.sound, NotificationSoundId::Pop);
    }

    #[test]
    fn preferences_patch_rejects_deferred_or_unknown_fields() {
        assert!(serde_json::from_value::<PreferencesSettingsPatch>(json!({
            "water": { "enabled": false }
        }))
        .is_err());
        assert!(serde_json::from_value::<PreferencesSettingsPatch>(json!({
            "updates": { "automatic": true }
        }))
        .is_err());
        assert!(serde_json::from_value::<PreferencesSettingsPatch>(json!({
            "notificationSounds": { "volume": 101 }
        }))
        .is_ok());

        let invalid = PreferencesSettingsPatch {
            general: None,
            notification_sounds: Some(NotificationSoundSettingsPatch {
                volume: Some(101),
                ..NotificationSoundSettingsPatch::default()
            }),
        };
        let directory = tempdir().unwrap();
        let state = SettingsState::new(
            SettingsStore::new(directory.path().join("settings.json")),
            SettingsDocument::default(),
        );
        assert!(matches!(
            state.update_preferences(invalid),
            Err(SettingsMutationError::Validation(_))
        ));
    }
}
